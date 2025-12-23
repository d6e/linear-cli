use serde::Deserialize;
use serde_json::json;
use tabled::Tabled;

use crate::client::LinearClient;
use crate::config::Config;
use crate::error::Result;
use crate::output::{self, format_date_only};
use crate::responses::Connection;
use crate::types::Cycle;

#[derive(Tabled)]
struct CycleRow {
    #[tabled(rename = "Number")]
    number: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Starts")]
    starts: String,
    #[tabled(rename = "Ends")]
    ends: String,
}

impl From<&Cycle> for CycleRow {
    fn from(cycle: &Cycle) -> Self {
        Self {
            number: cycle.number.to_string(),
            name: cycle.name.clone().unwrap_or_default(),
            starts: format_date_only(&cycle.starts_at),
            ends: format_date_only(&cycle.ends_at),
        }
    }
}

const LIST_CYCLES_QUERY: &str = r#"
query ListCycles($filter: CycleFilter) {
    cycles(filter: $filter) {
        nodes {
            id
            name
            number
            startsAt
            endsAt
        }
    }
}
"#;

#[derive(Deserialize)]
struct CyclesResponse {
    cycles: Connection<Cycle>,
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

    let response: CyclesResponse = client.query(LIST_CYCLES_QUERY, variables).await?;

    output::print_table(
        &response.cycles.nodes,
        |cycle| CycleRow::from(cycle),
        |cycle| {
            format!(
                "{} | {}",
                cycle.number,
                cycle.name.as_deref().unwrap_or("-")
            )
        },
    );

    Ok(())
}
