use serde::{Deserialize, Serialize};
use serde_json::json;
use tabled::Tabled;

use crate::cli::CycleListArgs;
use crate::client::LinearClient;
use crate::config::Config;
use crate::error::{LinearError, Result};
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

const GET_CYCLE_QUERY: &str = r#"
query GetCycle($id: String!) {
    cycle(id: $id) {
        id
        name
        number
        startsAt
        endsAt
        team {
            id
            key
            name
        }
    }
}
"#;

#[derive(Deserialize)]
struct CyclesResponse {
    cycles: Connection<Cycle>,
}

#[derive(Deserialize)]
struct CycleResponse {
    cycle: Option<CycleWithTeam>,
}

#[derive(Deserialize, Serialize)]
struct CycleWithTeam {
    #[allow(dead_code)]
    id: String,
    name: Option<String>,
    number: i32,
    #[serde(rename = "startsAt")]
    starts_at: String,
    #[serde(rename = "endsAt")]
    ends_at: String,
    team: Team,
}

#[derive(Deserialize, Serialize)]
struct Team {
    #[allow(dead_code)]
    id: String,
    key: String,
    name: String,
}

pub async fn list(client: &LinearClient, config: &Config, args: CycleListArgs) -> Result<()> {
    let team_key = config.resolve_team(args.team.as_deref());

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

pub async fn view(client: &LinearClient, id: &str) -> Result<()> {
    let variables = json!({ "id": id });
    let response: CycleResponse = client.query(GET_CYCLE_QUERY, Some(variables)).await?;

    let cycle = response
        .cycle
        .ok_or_else(|| LinearError::CycleNotFound(id.to_string()))?;

    output::print_item(&cycle, |cycle| {
        use colored::Colorize;

        let default_name = format!("Cycle {}", cycle.number);
        let name = cycle.name.as_deref().unwrap_or(&default_name);
        println!("{}", name.bold());
        println!();

        println!("Number: {}", cycle.number);
        println!("Team:   {} ({})", cycle.team.name, cycle.team.key);
        println!("Starts: {}", format_date_only(&cycle.starts_at));
        println!("Ends:   {}", format_date_only(&cycle.ends_at));
    });

    Ok(())
}
