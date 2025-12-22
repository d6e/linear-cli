use serde::Deserialize;
use serde_json::json;
use tabled::Tabled;

use crate::client::LinearClient;
use crate::config::Config;
use crate::error::Result;
use crate::output::{self, format_date_only};
use crate::responses::Connection;
use crate::types::Cycle;

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

#[derive(Tabled)]
struct CycleRow {
    #[tabled(rename = "Number")]
    number: i32,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Starts")]
    starts_at: String,
    #[tabled(rename = "Ends")]
    ends_at: String,
}

impl From<&Cycle> for CycleRow {
    fn from(cycle: &Cycle) -> Self {
        Self {
            number: cycle.number,
            name: cycle.name.clone().unwrap_or_default(),
            starts_at: format_date_only(&cycle.starts_at),
            ends_at: format_date_only(&cycle.ends_at),
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

    let response: CyclesResponse = client.query(LIST_CYCLES_QUERY, variables).await?;

    output::print_table(&response.cycles.nodes, |c| CycleRow::from(c));

    Ok(())
}
