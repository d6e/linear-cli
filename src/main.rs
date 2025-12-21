mod cli;
mod client;
mod commands;
mod config;
mod error;
mod types;

use clap::Parser;

use cli::{Cli, Commands, IssueCommands};
use client::LinearClient;
use config::Config;
use error::Result;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load()?;
    let client = LinearClient::new(config.api_key()?);

    match cli.command {
        Commands::Teams => {
            commands::teams::list(&client).await?;
        }
        Commands::Projects { team } => {
            commands::projects::list(&client, &config, team).await?;
        }
        Commands::Cycles { team } => {
            commands::cycles::list(&client, &config, team).await?;
        }
        Commands::Issues(args) => {
            commands::issues::list(&client, &config, args).await?;
        }
        Commands::Issue { action } => match action {
            IssueCommands::List(args) => {
                commands::issues::list(&client, &config, args).await?;
            }
            IssueCommands::Show { id } => {
                commands::issues::show(&client, &id).await?;
            }
            IssueCommands::Create(args) => {
                commands::issues::create(&client, &config, args).await?;
            }
            IssueCommands::Update(args) => {
                commands::issues::update(&client, args).await?;
            }
        },
    }

    Ok(())
}
