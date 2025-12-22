use serde::{Deserialize, Serialize};
use serde_json::json;
use tabled::Tabled;

use crate::client::LinearClient;
use crate::config::Config;
use crate::error::Result;
use crate::output::{self, status_colored, truncate};
use crate::responses::Connection;

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

#[derive(Tabled)]
struct LabelRow {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Description")]
    description: String,
    #[tabled(rename = "ID")]
    id: String,
}

impl From<&Label> for LabelRow {
    fn from(label: &Label) -> Self {
        Self {
            name: status_colored(&label.name, Some(&label.color)),
            description: truncate(label.description.as_deref().unwrap_or(""), 40),
            id: label.id.clone(),
        }
    }
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

    output::print_table(&labels, |l| LabelRow::from(l));

    Ok(())
}
