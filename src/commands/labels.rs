use serde::{Deserialize, Serialize};
use serde_json::json;
use tabled::Tabled;

use crate::client::LinearClient;
use crate::config::Config;
use crate::error::{LinearError, Result};
use crate::output::{self, is_json_output, status_colored, truncate};
use crate::responses::Connection;

#[derive(Tabled)]
struct LabelRow {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Description")]
    description: String,
    #[tabled(rename = "ID")]
    id: String,
}

impl LabelRow {
    fn from_label(label: &Label) -> Self {
        Self {
            name: if is_json_output() {
                label.name.clone()
            } else {
                status_colored(&label.name, Some(&label.color))
            },
            description: truncate(label.description.as_deref().unwrap_or(""), 40),
            id: label.id.clone(),
        }
    }
}

const LIST_LABELS_QUERY: &str = r#"
query ListLabels($filter: IssueLabelFilter) {
    issueLabels(filter: $filter) {
        nodes {
            id
            name
            color
            description
        }
    }
}
"#;

#[derive(Deserialize)]
struct LabelsResponse {
    #[serde(rename = "issueLabels")]
    issue_labels: Connection<Label>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Label {
    pub id: String,
    pub name: String,
    pub color: String,
    pub description: Option<String>,
}

pub async fn list(client: &LinearClient, config: &Config, team: Option<String>) -> Result<()> {
    let team_key = config.resolve_team(team.as_deref());

    let variables = team_key.map(|key| {
        json!({
            "filter": {
                "team": {
                    "key": { "eq": key }
                }
            }
        })
    });

    let response: LabelsResponse = client.query(LIST_LABELS_QUERY, variables).await?;
    let labels = response.issue_labels.nodes;

    if labels.is_empty() {
        output::print_message("No labels found");
        return Ok(());
    }

    output::print_table(&labels, LabelRow::from_label, |label| label.name.clone());

    Ok(())
}

const GET_ISSUE_LABELS_QUERY: &str = r#"
query GetIssue($id: String!) {
    issue(id: $id) {
        id
        identifier
        labels {
            nodes {
                id
                name
                color
                description
            }
        }
    }
}
"#;

#[derive(Deserialize)]
struct IssueLabelResponse {
    issue: Option<IssueLabelData>,
}

#[derive(Deserialize)]
struct IssueLabelData {
    #[allow(dead_code)]
    id: String,
    identifier: String,
    labels: Connection<Label>,
}

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

#[derive(Deserialize)]
struct UpdateIssueResponse {
    #[serde(rename = "issueUpdate")]
    issue_update: IssueUpdateResult,
}

#[derive(Deserialize)]
struct IssueUpdateResult {
    success: bool,
}

/// List labels on an issue
pub async fn list_for_issue(client: &LinearClient, id: &str) -> Result<()> {
    let variables = json!({ "id": id });
    let response: IssueLabelResponse = client.query(GET_ISSUE_LABELS_QUERY, Some(variables)).await?;

    let issue = response
        .issue
        .ok_or_else(|| LinearError::IssueNotFound(id.to_string()))?;

    let labels = issue.labels.nodes;

    if labels.is_empty() {
        output::print_message(&format!("No labels on {}", issue.identifier));
        return Ok(());
    }

    output::print_table(&labels, LabelRow::from_label, |label| label.name.clone());

    Ok(())
}

/// Resolve a label name to its ID (case-insensitive)
pub async fn resolve_label_id(client: &LinearClient, name: &str) -> Result<String> {
    let response: LabelsResponse = client.query(LIST_LABELS_QUERY, None).await?;
    let name_lower = name.to_lowercase();

    response
        .issue_labels
        .nodes
        .iter()
        .find(|l| l.name.to_lowercase() == name_lower)
        .map(|l| l.id.clone())
        .ok_or_else(|| LinearError::LabelNotFound(name.to_string()))
}

/// Resolve multiple label names to IDs
pub async fn resolve_label_ids(client: &LinearClient, names: &[String]) -> Result<Vec<String>> {
    if names.is_empty() {
        return Ok(Vec::new());
    }

    let response: LabelsResponse = client.query(LIST_LABELS_QUERY, None).await?;
    let mut ids = Vec::with_capacity(names.len());

    for name in names {
        let name_lower = name.to_lowercase();
        let label = response
            .issue_labels
            .nodes
            .iter()
            .find(|l| l.name.to_lowercase() == name_lower)
            .ok_or_else(|| LinearError::LabelNotFound(name.to_string()))?;
        ids.push(label.id.clone());
    }

    Ok(ids)
}

/// Get current label IDs for an issue
pub async fn get_issue_label_ids(client: &LinearClient, id: &str) -> Result<Vec<String>> {
    let variables = json!({ "id": id });
    let response: IssueLabelResponse = client.query(GET_ISSUE_LABELS_QUERY, Some(variables)).await?;

    let issue = response
        .issue
        .ok_or_else(|| LinearError::IssueNotFound(id.to_string()))?;

    Ok(issue.labels.nodes.iter().map(|l| l.id.clone()).collect())
}

/// Add a label to an issue
pub async fn add_label(client: &LinearClient, id: &str, label_name: &str) -> Result<()> {
    let label_id = resolve_label_id(client, label_name).await?;
    let mut current_ids = get_issue_label_ids(client, id).await?;

    if current_ids.contains(&label_id) {
        output::print_message(&format!("Issue already has label '{}'", label_name));
        return Ok(());
    }

    current_ids.push(label_id);

    let variables = json!({
        "id": id,
        "input": {
            "labelIds": current_ids
        }
    });

    let response: UpdateIssueResponse = client.query(UPDATE_ISSUE_MUTATION, Some(variables)).await?;

    if response.issue_update.success {
        output::print_message(&format!("Added label '{}' to issue", label_name));
    }

    Ok(())
}

/// Remove a label from an issue
pub async fn remove_label(client: &LinearClient, id: &str, label_name: &str) -> Result<()> {
    let label_id = resolve_label_id(client, label_name).await?;
    let current_ids = get_issue_label_ids(client, id).await?;

    if !current_ids.contains(&label_id) {
        output::print_message(&format!("Issue does not have label '{}'", label_name));
        return Ok(());
    }

    let new_ids: Vec<_> = current_ids.into_iter().filter(|id| id != &label_id).collect();

    let variables = json!({
        "id": id,
        "input": {
            "labelIds": new_ids
        }
    });

    let response: UpdateIssueResponse = client.query(UPDATE_ISSUE_MUTATION, Some(variables)).await?;

    if response.issue_update.success {
        output::print_message(&format!("Removed label '{}' from issue", label_name));
    }

    Ok(())
}
