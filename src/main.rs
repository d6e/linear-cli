mod cache;
mod cli;
mod client;
mod commands;
mod config;
mod error;
mod output;
mod responses;
mod types;

use std::io;

use clap::{CommandFactory, Parser};
use clap_complete::generate;

use cli::{Cli, Commands, CycleCommands, IssueCommands, ImageCommands, AttachmentCommands};
use client::LinearClient;
use config::Config;
use error::Result;
use std::error::Error;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {e}");

        // Show error chain if verbose flag was passed
        if std::env::args().any(|arg| arg == "--verbose" || arg == "-v") {
            let mut source = e.source();
            while let Some(cause) = source {
                eprintln!("Caused by: {cause}");
                source = std::error::Error::source(cause);
            }
        }

        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    // Set global output format
    output::set_format(cli.output_format());
    output::set_quiet(cli.quiet);

    match cli.command {
        // Commands that don't require config/client
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "linear", &mut io::stdout());
        }
        Commands::Init => {
            commands::init::run().await?;
        }
        // Commands that require config and client
        command => {
            let config = Config::load()?;
            let client = LinearClient::new(config.api_key()?);

            match command {
                Commands::Teams => {
                    commands::teams::list(&client).await?;
                }
                Commands::Projects { team } => {
                    commands::projects::list(&client, &config, team).await?;
                }
                Commands::Cycles(args) => {
                    commands::cycles::list(&client, &config, args).await?;
                }
                Commands::Cycle { action } => match action {
                    CycleCommands::List(args) => {
                        commands::cycles::list(&client, &config, args).await?;
                    }
                    CycleCommands::View { id } => {
                        commands::cycles::view(&client, &id).await?;
                    }
                },
                Commands::Issues(args) => {
                    commands::issues::list(&client, &config, args).await?;
                }
                Commands::Labels { team } => {
                    commands::labels::list(&client, &config, team).await?;
                }
                Commands::Issue { action } => match action {
                    IssueCommands::List(args) => {
                        commands::issues::list(&client, &config, args).await?;
                    }
                    IssueCommands::View(args) => {
                        commands::issues::view(&client, args).await?;
                    }
                    IssueCommands::Download(args) => {
                        commands::issues::download_all(&client, args).await?;
                    }
                    IssueCommands::Images { action } => match action {
                        ImageCommands::Download(args) => {
                            commands::images::download_images_command(&client, args).await?;
                        }
                    },
                    IssueCommands::Create(args) => {
                        commands::issues::create(&client, &config, args).await?;
                    }
                    IssueCommands::Update(args) => {
                        commands::issues::update(&client, args).await?;
                    }
                    IssueCommands::Close { id } => {
                        commands::issues::close(&client, &id).await?;
                    }
                    IssueCommands::Attachments { action } => match action {
                        AttachmentCommands::List { id } => {
                            commands::attachments::list(&client, &id).await?;
                        }
                        AttachmentCommands::Download(args) => {
                            commands::attachments::download(&client, args).await?;
                        }
                        AttachmentCommands::Attach(args) => {
                            commands::attachments::attach_url(&client, args).await?;
                        }
                        AttachmentCommands::Upload(args) => {
                            commands::attachments::upload_file(&client, args).await?;
                        }
                    },
                    IssueCommands::Comments { id } => {
                        commands::comments::list(&client, &id).await?;
                    }
                    IssueCommands::Comment(args) => {
                        commands::comments::add(&client, args).await?;
                    }
                    IssueCommands::Relations { id } => {
                        commands::relations::list(&client, &id).await?;
                    }
                    IssueCommands::Relate(args) => {
                        commands::relations::relate(&client, args).await?;
                    }
                    IssueCommands::Unrelate { source, target } => {
                        commands::relations::unrelate(&client, &source, &target).await?;
                    }
                    IssueCommands::Parent { id, parent_id } => {
                        commands::relations::set_parent(&client, &id, &parent_id).await?;
                    }
                    IssueCommands::Unparent { id } => {
                        commands::relations::remove_parent(&client, &id).await?;
                    }
                },
                Commands::Completions { .. } | Commands::Init => {
                    // Already handled above
                }
            }
        }
    }

    Ok(())
}
