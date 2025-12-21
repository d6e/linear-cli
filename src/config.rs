use std::path::PathBuf;

use directories::ProjectDirs;
use serde::Deserialize;

use crate::error::{LinearError, Result};

#[derive(Deserialize, Default)]
pub struct Config {
    pub api_key: Option<String>,
    pub default_team: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            return Ok(Config::default());
        }

        let contents =
            std::fs::read_to_string(&config_path).map_err(|e| LinearError::ConfigRead {
                path: config_path.clone(),
                source: e,
            })?;

        toml::from_str(&contents).map_err(|e| LinearError::ConfigParse {
            path: config_path,
            source: e,
        })
    }

    pub fn config_path() -> Result<PathBuf> {
        ProjectDirs::from("", "", "linear")
            .map(|dirs| dirs.config_dir().join("config.toml"))
            .ok_or(LinearError::NoConfigDir)
    }

    /// Get API key with env var taking precedence over config file
    pub fn api_key(&self) -> Result<String> {
        if let Ok(key) = std::env::var("LINEAR_API_KEY") {
            return Ok(key);
        }

        self.api_key.clone().ok_or(LinearError::MissingApiKey)
    }

    /// Get team, preferring explicit argument over default
    pub fn resolve_team(&self, explicit: Option<&str>) -> Option<String> {
        explicit
            .map(String::from)
            .or_else(|| self.default_team.clone())
    }
}
