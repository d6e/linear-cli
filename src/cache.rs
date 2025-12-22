use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

use crate::config::Config;

const CACHE_TTL_SECS: u64 = 3600; // 1 hour

#[derive(Serialize, Deserialize, Default)]
pub struct Cache {
    teams: HashMap<String, CachedTeam>,
    #[serde(default)]
    timestamp: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CachedTeam {
    pub id: String,
    pub key: String,
    pub name: String,
}

impl Cache {
    pub fn load() -> Self {
        let path = match Self::cache_path() {
            Ok(p) => p,
            Err(_) => {
                eprintln!("Warning: Could not determine cache path");
                return Self::default();
            }
        };

        if !path.exists() {
            return Self::default();
        }

        let contents = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: Could not read cache file: {e}");
                return Self::default();
            }
        };

        let cache: Self = match serde_json::from_str(&contents) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: Could not parse cache file: {e}");
                return Self::default();
            }
        };

        // Check if cache is expired
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if now - cache.timestamp > CACHE_TTL_SECS {
            return Self::default();
        }

        cache
    }

    pub fn save(&self) {
        let path = match Self::cache_path() {
            Ok(p) => p,
            Err(_) => {
                eprintln!("Warning: Could not determine cache path for saving");
                return;
            }
        };

        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!("Warning: Could not create cache directory: {e}");
                return;
            }
        }

        let contents = match serde_json::to_string_pretty(self) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: Could not serialize cache: {e}");
                return;
            }
        };

        if let Err(e) = std::fs::write(&path, contents) {
            eprintln!("Warning: Could not write cache file: {e}");
        }
    }

    fn cache_path() -> Result<PathBuf, ()> {
        Config::config_path()
            .map(|p| p.with_file_name("cache.json"))
            .map_err(|_| ())
    }

    pub fn set_team(&mut self, team: CachedTeam) {
        self.teams.insert(team.key.clone(), team);
        self.timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
    }

    pub fn get_team_id(&self, key: &str) -> Option<String> {
        self.teams.get(key).map(|t| t.id.clone())
    }
}
