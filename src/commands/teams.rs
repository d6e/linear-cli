use serde::Deserialize;
use tabled::{Table, Tabled, settings::Style};

use crate::client::LinearClient;
use crate::error::Result;
use crate::types::Team;

const LIST_TEAMS_QUERY: &str = r#"
query ListTeams {
    teams {
        nodes {
            id
            key
            name
        }
    }
}
"#;

#[derive(Deserialize)]
struct TeamsResponse {
    teams: TeamsConnection,
}

#[derive(Deserialize)]
struct TeamsConnection {
    nodes: Vec<Team>,
}

#[derive(Tabled)]
struct TeamRow {
    #[tabled(rename = "Key")]
    key: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "ID")]
    id: String,
}

impl From<&Team> for TeamRow {
    fn from(team: &Team) -> Self {
        Self {
            key: team.key.clone(),
            name: team.name.clone(),
            id: team.id.clone(),
        }
    }
}

pub async fn list(client: &LinearClient) -> Result<()> {
    let response: TeamsResponse = client.query(LIST_TEAMS_QUERY, None).await?;

    let rows: Vec<TeamRow> = response.teams.nodes.iter().map(TeamRow::from).collect();
    let table = Table::new(rows).with(Style::rounded()).to_string();
    println!("{table}");

    Ok(())
}
