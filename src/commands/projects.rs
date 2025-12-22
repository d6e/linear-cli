use serde::Deserialize;
use serde_json::json;
use tabled::Tabled;

use crate::client::LinearClient;
use crate::config::Config;
use crate::error::Result;
use crate::output;
use crate::responses::Connection;
use crate::types::Project;

const LIST_PROJECTS_QUERY: &str = r#"
query ListProjects($filter: ProjectFilter) {
    projects(filter: $filter) {
        nodes {
            id
            name
            state
        }
    }
}
"#;

#[derive(Deserialize)]
struct ProjectsResponse {
    projects: Connection<Project>,
}

#[derive(Tabled)]
struct ProjectRow {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "State")]
    state: String,
    #[tabled(rename = "ID")]
    id: String,
}

impl From<&Project> for ProjectRow {
    fn from(project: &Project) -> Self {
        Self {
            name: project.name.clone(),
            state: project.state.clone().unwrap_or_default(),
            id: project.id.clone(),
        }
    }
}

pub async fn list(client: &LinearClient, config: &Config, team: Option<String>) -> Result<()> {
    let team_key = config.resolve_team(team.as_deref());

    let variables = team_key.map(|key| {
        json!({
            "filter": {
                "accessibleTeams": {
                    "key": { "eq": key }
                }
            }
        })
    });

    let response: ProjectsResponse = client.query(LIST_PROJECTS_QUERY, variables).await?;

    output::print_table(&response.projects.nodes, |p| ProjectRow::from(p));

    Ok(())
}
