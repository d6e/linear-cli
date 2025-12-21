use std::io::{self, Write};

use crate::config::Config;
use crate::error::{LinearError, Result};

pub async fn run() -> Result<()> {
    let config_path = Config::config_path()?;

    if config_path.exists() {
        print!(
            "Config file already exists at {}. Overwrite? [y/N] ",
            config_path.display()
        );
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    println!("Linear CLI Configuration");
    println!("========================\n");

    // Get API key
    print!("Enter your Linear API key (create one at https://linear.app/settings/api): ");
    io::stdout().flush().unwrap();

    let mut api_key = String::new();
    io::stdin().read_line(&mut api_key).unwrap();
    let api_key = api_key.trim();

    if api_key.is_empty() {
        return Err(LinearError::MissingApiKey);
    }

    // Get default team (optional)
    print!("Enter default team key (e.g., ENG) [optional]: ");
    io::stdout().flush().unwrap();

    let mut default_team = String::new();
    io::stdin().read_line(&mut default_team).unwrap();
    let default_team = default_team.trim();

    // Create config directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| LinearError::ConfigRead {
            path: config_path.clone(),
            source: e,
        })?;
    }

    // Write config file
    let mut config_content = format!("api_key = \"{api_key}\"\n");
    if !default_team.is_empty() {
        config_content.push_str(&format!("default_team = \"{default_team}\"\n"));
    }

    std::fs::write(&config_path, config_content).map_err(|e| LinearError::ConfigRead {
        path: config_path.clone(),
        source: e,
    })?;

    println!("\nConfig saved to {}", config_path.display());
    println!("You can now use 'linear' commands!");

    Ok(())
}
