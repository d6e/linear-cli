use serde::Deserialize;
use serde_json::json;
use tabled::{Table, Tabled, settings::Style};

use crate::cli::{IssueCreateArgs, IssueListArgs, IssueUpdateArgs};
use crate::client::LinearClient;
use crate::config::Config;
use crate::error::{LinearError, Result};
use crate::types::Issue;

const LIST_ISSUES_QUERY: &str = r#"
query ListIssues($filter: IssueFilter, $first: Int) {
    issues(filter: $filter, first: $first) {
        nodes {
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
    }
}
"#;

const GET_ISSUE_QUERY: &str = r#"
query GetIssue($id: String!) {
    issue(id: $id) {
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
}
"#;

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
struct CreatedIssue {
    identifier: String,
    title: String,
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
struct ViewerResponse {
    viewer: Viewer,
}

#[derive(Deserialize)]
struct Viewer {
    id: String,
}

#[derive(Deserialize)]
struct TeamsResponse {
    teams: TeamsConnection,
}

#[derive(Deserialize)]
struct TeamsConnection {
    nodes: Vec<TeamNode>,
}

#[derive(Deserialize)]
struct TeamNode {
    id: String,
}

#[derive(Deserialize)]
struct WorkflowStatesResponse {
    #[serde(rename = "workflowStates")]
    workflow_states: WorkflowStatesConnection,
}

#[derive(Deserialize)]
struct WorkflowStatesConnection {
    nodes: Vec<WorkflowStateNode>,
}

#[derive(Deserialize)]
struct WorkflowStateNode {
    id: String,
    name: String,
}

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
        Self {
            id: issue.identifier.clone(),
            title: truncate(&issue.title, 50),
            status: issue
                .state
                .as_ref()
                .map(|s| s.name.clone())
                .unwrap_or_default(),
            priority: priority_label(issue.priority),
            assignee: issue
                .assignee
                .as_ref()
                .map(|u| u.name.clone())
                .unwrap_or_default(),
        }
    }
}

fn priority_label(priority: i32) -> String {
    match priority {
        0 => "None".to_string(),
        1 => "Urgent".to_string(),
        2 => "High".to_string(),
        3 => "Medium".to_string(),
        4 => "Low".to_string(),
        _ => format!("P{priority}"),
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}

pub async fn list(client: &LinearClient, config: &Config, args: IssueListArgs) -> Result<()> {
    let mut filter = serde_json::Map::new();

    // Team filter
    if let Some(team_key) = config.resolve_team(args.team.as_deref()) {
        filter.insert(
            "team".to_string(),
            json!({ "key": { "eq": team_key } }),
        );
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

    // Mine filter
    if args.mine {
        let viewer: ViewerResponse = client.query(GET_VIEWER_QUERY, None).await?;
        filter.insert(
            "assignee".to_string(),
            json!({ "id": { "eq": viewer.viewer.id } }),
        );
    }

    let variables = json!({
        "filter": filter,
        "first": args.limit
    });

    let response: IssuesResponse = client.query(LIST_ISSUES_QUERY, Some(variables)).await?;

    let rows: Vec<IssueRow> = response.issues.nodes.iter().map(IssueRow::from).collect();
    let table = Table::new(rows).with(Style::rounded()).to_string();
    println!("{table}");

    Ok(())
}

pub async fn show(client: &LinearClient, id: &str) -> Result<()> {
    let variables = json!({ "id": id });
    let response: IssueResponse = client.query(GET_ISSUE_QUERY, Some(variables)).await?;

    let issue = response
        .issue
        .ok_or_else(|| LinearError::IssueNotFound(id.to_string()))?;

    println!("{} - {}", issue.identifier, issue.title);
    println!();

    if let Some(desc) = &issue.description {
        println!("{desc}");
        println!();
    }

    println!("Team:     {}", issue.team.name);
    println!(
        "Status:   {}",
        issue.state.as_ref().map(|s| &s.name[..]).unwrap_or("-")
    );
    println!("Priority: {}", priority_label(issue.priority));
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
            cycle.name.as_deref().unwrap_or(&format!("Cycle {}", cycle.number))
        );
    }

    Ok(())
}

pub async fn create(client: &LinearClient, config: &Config, args: IssueCreateArgs) -> Result<()> {
    let team_key = config
        .resolve_team(args.team.as_deref())
        .ok_or(LinearError::NoTeam)?;

    // Get team ID from key
    let team_response: TeamsResponse = client
        .query(GET_TEAM_BY_KEY_QUERY, Some(json!({ "key": team_key })))
        .await?;

    let team_id = team_response
        .teams
        .nodes
        .first()
        .map(|t| t.id.clone())
        .ok_or_else(|| LinearError::TeamNotFound(team_key))?;

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
    let response: CreateIssueResponse = client
        .query(CREATE_ISSUE_MUTATION, Some(variables))
        .await?;

    if response.issue_create.success {
        if let Some(issue) = response.issue_create.issue {
            println!("Created {} - {}", issue.identifier, issue.title);
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

        let state_id = states_response
            .workflow_states
            .nodes
            .iter()
            .find(|s| s.name.to_lowercase().contains(&status_name.to_lowercase()))
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
        }
    }

    if input.is_empty() {
        println!("No updates specified");
        return Ok(());
    }

    let variables = json!({
        "id": args.id,
        "input": input
    });

    let response: UpdateIssueResponse = client
        .query(UPDATE_ISSUE_MUTATION, Some(variables))
        .await?;

    if response.issue_update.success {
        if let Some(issue) = response.issue_update.issue {
            println!("Updated {} - {}", issue.identifier, issue.title);
        }
    }

    Ok(())
}
