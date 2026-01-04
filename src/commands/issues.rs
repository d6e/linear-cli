use serde::Deserialize;
use serde_json::json;
use tabled::Tabled;

use crate::cache::{Cache, CachedTeam};
use crate::cli::{IssueCreateArgs, IssueListArgs, IssueUpdateArgs};
use crate::client::LinearClient;
use crate::config::Config;
use crate::error::{LinearError, Result};
use crate::output::{self, format_date, is_json_output, status_colored, truncate};
use crate::responses::{
    Connection, CreatedIssue, PageInfo, TeamNode, ViewerResponse, WorkflowStateNode,
};
use crate::types::Issue;

#[derive(Tabled)]
struct IssueRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Title")]
    title: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Priority")]
    priority: String,
    #[tabled(rename = "Assignee")]
    assignee: String,
}

impl From<&Issue> for IssueRow {
    fn from(issue: &Issue) -> Self {
        let (status_name, status_color) = issue
            .state
            .as_ref()
            .map(|s| (s.name.clone(), Some(s.color.clone())))
            .unwrap_or_default();

        Self {
            id: issue.identifier.clone(),
            title: truncate(&issue.title, 50),
            status: if is_json_output() {
                status_name
            } else {
                status_colored(&status_name, status_color.as_deref())
            },
            priority: if is_json_output() {
                issue.priority.to_string()
            } else {
                issue.priority.colored()
            },
            assignee: issue
                .assignee
                .as_ref()
                .map(|u| u.name.clone())
                .unwrap_or_default(),
        }
    }
}

const ISSUE_FIELDS_FRAGMENT: &str = r#"
fragment IssueFields on Issue {
    id
    identifier
    title
    description
    priority
    state {
        id
        name
        color
    }
    assignee {
        id
        name
        email
    }
    team {
        id
        key
        name
    }
    project {
        id
        name
        state
    }
    cycle {
        id
        name
        number
        startsAt
        endsAt
    }
    createdAt
    updatedAt
}
"#;

const LIST_ISSUES_QUERY: &str = const_format::concatcp!(
    r#"
query ListIssues($filter: IssueFilter, $first: Int, $after: String) {
    issues(filter: $filter, first: $first, after: $after) {
        nodes {
            ...IssueFields
        }
        pageInfo {
            hasNextPage
            endCursor
        }
    }
}
"#,
    ISSUE_FIELDS_FRAGMENT
);

const GET_ISSUE_QUERY: &str = const_format::concatcp!(
    r#"
query GetIssue($id: String!) {
    issue(id: $id) {
        ...IssueFields
    }
}
"#,
    ISSUE_FIELDS_FRAGMENT
);

const CREATE_ISSUE_MUTATION: &str = r#"
mutation CreateIssue($input: IssueCreateInput!) {
    issueCreate(input: $input) {
        success
        issue {
            id
            identifier
            title
        }
    }
}
"#;

const UPDATE_ISSUE_MUTATION: &str = r#"
mutation UpdateIssue($id: String!, $input: IssueUpdateInput!) {
    issueUpdate(id: $id, input: $input) {
        success
        issue {
            id
            identifier
            title
        }
    }
}
"#;

const GET_VIEWER_QUERY: &str = r#"
query Viewer {
    viewer {
        id
    }
}
"#;

const GET_TEAM_BY_KEY_QUERY: &str = r#"
query GetTeam($key: String!) {
    teams(filter: { key: { eq: $key } }) {
        nodes {
            id
            key
            name
        }
    }
}
"#;

const GET_STATES_QUERY: &str = r#"
query GetStates($teamId: String!) {
    workflowStates(filter: { team: { id: { eq: $teamId } } }) {
        nodes {
            id
            name
            type
        }
    }
}
"#;

#[derive(Deserialize)]
struct IssuesResponse {
    issues: IssuesConnection,
}

#[derive(Deserialize)]
struct IssuesConnection {
    nodes: Vec<Issue>,
    #[serde(rename = "pageInfo")]
    page_info: PageInfo,
}

#[derive(Deserialize)]
struct IssueResponse {
    issue: Option<Issue>,
}

#[derive(Deserialize)]
struct CreateIssueResponse {
    #[serde(rename = "issueCreate")]
    issue_create: IssueCreateResult,
}

#[derive(Deserialize)]
struct IssueCreateResult {
    success: bool,
    issue: Option<CreatedIssue>,
}

#[derive(Deserialize)]
struct UpdateIssueResponse {
    #[serde(rename = "issueUpdate")]
    issue_update: IssueUpdateResult,
}

#[derive(Deserialize)]
struct IssueUpdateResult {
    success: bool,
    issue: Option<CreatedIssue>,
}

#[derive(Deserialize)]
struct TeamsResponse {
    teams: Connection<TeamNode>,
}

#[derive(Deserialize)]
struct WorkflowStatesResponse {
    #[serde(rename = "workflowStates")]
    workflow_states: Connection<WorkflowStateNode>,
}

pub async fn list(client: &LinearClient, config: &Config, args: IssueListArgs) -> Result<()> {
    let mut filter = serde_json::Map::new();

    // Team filter
    if let Some(team_key) = config.resolve_team(args.team.as_deref()) {
        filter.insert("team".to_string(), json!({ "key": { "eq": team_key } }));
    }

    // Status filter
    if let Some(status) = &args.status {
        filter.insert(
            "state".to_string(),
            json!({ "name": { "containsIgnoreCase": status } }),
        );
    }

    // Project filter
    if let Some(project) = &args.project {
        filter.insert(
            "project".to_string(),
            json!({ "name": { "containsIgnoreCase": project } }),
        );
    }

    // Label filter
    if let Some(label) = &args.label {
        filter.insert(
            "labels".to_string(),
            json!({ "name": { "containsIgnoreCase": label } }),
        );
    }

    // Cycle filter
    if let Some(cycle) = &args.cycle {
        filter.insert(
            "cycle".to_string(),
            json!({ "name": { "containsIgnoreCase": cycle } }),
        );
    }

    // Mine filter
    if args.mine {
        let viewer: ViewerResponse = client.query(GET_VIEWER_QUERY, None).await?;
        filter.insert(
            "assignee".to_string(),
            json!({ "id": { "eq": viewer.viewer.id } }),
        );
    }

    // Pagination support
    let page_size = if args.all { 100 } else { args.limit.min(250) };
    let mut all_issues: Vec<Issue> = Vec::new();
    let mut cursor: Option<String> = None;

    loop {
        let mut variables = json!({
            "filter": filter.clone(),
            "first": page_size
        });

        if let Some(ref c) = cursor {
            variables["after"] = json!(c);
        }

        let response: IssuesResponse = client.query(LIST_ISSUES_QUERY, Some(variables)).await?;
        all_issues.extend(response.issues.nodes);

        if !args.all || !response.issues.page_info.has_next_page {
            break;
        }

        cursor = response.issues.page_info.end_cursor;
        if cursor.is_none() {
            break;
        }
    }

    output::print_table(
        &all_issues,
        |issue| IssueRow::from(issue),
        |issue| {
            let status = issue.state.as_ref().map(|s| s.name.as_str()).unwrap_or("-");
            format!(
                "{} | {} | {}",
                issue.identifier,
                truncate(&issue.title, 50),
                status
            )
        },
    );

    Ok(())
}

pub async fn show(client: &LinearClient, id: &str) -> Result<()> {
    let variables = json!({ "id": id });
    let response: IssueResponse = client.query(GET_ISSUE_QUERY, Some(variables)).await?;

    let issue = response
        .issue
        .ok_or_else(|| LinearError::IssueNotFound(id.to_string()))?;

    output::print_item(&issue, |issue| {
        use colored::Colorize;

        println!("{} - {}", issue.identifier.bold(), issue.title);
        println!();

        if let Some(desc) = &issue.description {
            println!("{desc}");
            println!();
        }

        println!("Team:     {}", issue.team.name);

        let (status_name, status_color) = issue
            .state
            .as_ref()
            .map(|s| (s.name.as_str(), Some(s.color.as_str())))
            .unwrap_or(("-", None));
        println!("Status:   {}", status_colored(status_name, status_color));

        println!("Priority: {}", issue.priority.colored());

        println!(
            "Assignee: {}",
            issue.assignee.as_ref().map(|u| &u.name[..]).unwrap_or("-")
        );

        if let Some(project) = &issue.project {
            println!("Project:  {}", project.name);
        }

        if let Some(cycle) = &issue.cycle {
            println!(
                "Cycle:    {}",
                cycle
                    .name
                    .as_deref()
                    .unwrap_or(&format!("Cycle {}", cycle.number))
            );
        }

        println!("Created:  {}", format_date(&issue.created_at));
        println!("Updated:  {}", format_date(&issue.updated_at));
    });

    Ok(())
}

pub async fn create(client: &LinearClient, config: &Config, args: IssueCreateArgs) -> Result<()> {
    let team_key = config
        .resolve_team(args.team.as_deref())
        .ok_or(LinearError::NoTeam)?;

    // Try to get team ID from cache first
    let mut cache = Cache::load();
    let team_id = if let Some(cached_id) = cache.get_team_id(&team_key) {
        cached_id
    } else {
        // Fetch from API and cache
        let team_response: TeamsResponse = client
            .query(GET_TEAM_BY_KEY_QUERY, Some(json!({ "key": team_key })))
            .await?;

        let team = team_response
            .teams
            .nodes
            .first()
            .ok_or_else(|| LinearError::TeamNotFound(team_key.clone()))?;

        cache.set_team(CachedTeam {
            id: team.id.clone(),
            key: team_key.clone(),
            name: "".to_string(), // We don't have the name in this response
        });
        cache.save();

        team.id.clone()
    };

    let mut input = serde_json::Map::new();
    input.insert("title".to_string(), json!(args.title));
    input.insert("teamId".to_string(), json!(team_id));

    if let Some(desc) = args.description {
        input.insert("description".to_string(), json!(desc));
    }
    if let Some(priority) = args.priority {
        input.insert("priority".to_string(), json!(priority));
    }

    let variables = json!({ "input": input });
    let response: CreateIssueResponse =
        client.query(CREATE_ISSUE_MUTATION, Some(variables)).await?;

    if response.issue_create.success {
        if let Some(issue) = response.issue_create.issue {
            output::print_message(&format!("Created {} - {}", issue.identifier, issue.title));
        }
    }

    Ok(())
}

pub async fn update(client: &LinearClient, args: IssueUpdateArgs) -> Result<()> {
    let mut input = serde_json::Map::new();

    if let Some(title) = args.title {
        input.insert("title".to_string(), json!(title));
    }
    if let Some(desc) = args.description {
        input.insert("description".to_string(), json!(desc));
    }
    if let Some(priority) = args.priority {
        input.insert("priority".to_string(), json!(priority));
    }

    // Handle status change - need to resolve name to ID
    if let Some(status_name) = &args.status {
        // First get the issue to find its team
        let issue_response: IssueResponse = client
            .query(GET_ISSUE_QUERY, Some(json!({ "id": args.id })))
            .await?;

        let issue = issue_response
            .issue
            .ok_or_else(|| LinearError::IssueNotFound(args.id.clone()))?;

        // Get workflow states for the team
        let states_response: WorkflowStatesResponse = client
            .query(GET_STATES_QUERY, Some(json!({ "teamId": issue.team.id })))
            .await?;

        let status_lower = status_name.to_lowercase();
        let state_id = states_response
            .workflow_states
            .nodes
            .iter()
            .find(|s| s.name.to_lowercase() == status_lower)
            .map(|s| s.id.clone());

        if let Some(id) = state_id {
            input.insert("stateId".to_string(), json!(id));
        }
    }

    // Handle assignee
    if let Some(assignee) = &args.assignee {
        if assignee == "me" {
            let viewer: ViewerResponse = client.query(GET_VIEWER_QUERY, None).await?;
            input.insert("assigneeId".to_string(), json!(viewer.viewer.id));
        } else {
            // Treat as user ID directly
            input.insert("assigneeId".to_string(), json!(assignee));
        }
    }

    if input.is_empty() {
        output::print_message("No updates specified");
        return Ok(());
    }

    let variables = json!({
        "id": args.id,
        "input": input
    });

    let response: UpdateIssueResponse =
        client.query(UPDATE_ISSUE_MUTATION, Some(variables)).await?;

    if response.issue_update.success {
        if let Some(issue) = response.issue_update.issue {
            output::print_message(&format!("Updated {} - {}", issue.identifier, issue.title));
        }
    }

    Ok(())
}

/// Close an issue by setting its status to a "done" state
pub async fn close(client: &LinearClient, id: &str) -> Result<()> {
    // First get the issue to find its team
    let issue_response: IssueResponse = client
        .query(GET_ISSUE_QUERY, Some(json!({ "id": id })))
        .await?;

    let issue = issue_response
        .issue
        .ok_or_else(|| LinearError::IssueNotFound(id.to_string()))?;

    // Get workflow states for the team
    let states_response: WorkflowStatesResponse = client
        .query(GET_STATES_QUERY, Some(json!({ "teamId": issue.team.id })))
        .await?;

    // Find a completed state using the state type
    let done_state = states_response
        .workflow_states
        .nodes
        .iter()
        .find(|s| s.state_type == "completed")
        .ok_or_else(|| {
            LinearError::WorkflowStateNotFound("No completed state found for team".to_string())
        })?;

    let variables = json!({
        "id": id,
        "input": {
            "stateId": done_state.id
        }
    });

    let response: UpdateIssueResponse =
        client.query(UPDATE_ISSUE_MUTATION, Some(variables)).await?;

    if response.issue_update.success {
        if let Some(updated_issue) = response.issue_update.issue {
            output::print_message(&format!(
                "Closed {} - {}",
                updated_issue.identifier, updated_issue.title
            ));
        }
    }

    Ok(())
}
