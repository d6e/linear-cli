use std::fmt;

use clap::ValueEnum;
use colored::Colorize;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Priority levels for issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Priority {
    /// No priority (0)
    None = 0,
    /// Urgent priority (1)
    Urgent = 1,
    /// High priority (2)
    High = 2,
    /// Medium priority (3)
    Medium = 3,
    /// Low priority (4)
    Low = 4,
}

impl Priority {
    /// Create Priority from an integer value.
    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => Priority::None,
            1 => Priority::Urgent,
            2 => Priority::High,
            3 => Priority::Medium,
            4 => Priority::Low,
            _ => Priority::None,
        }
    }

    /// Get the integer value.
    pub fn as_i32(self) -> i32 {
        self as i32
    }

    /// Get the label for this priority.
    pub fn label(self) -> &'static str {
        match self {
            Priority::None => "None",
            Priority::Urgent => "Urgent",
            Priority::High => "High",
            Priority::Medium => "Medium",
            Priority::Low => "Low",
        }
    }

    /// Get the colored label for terminal output.
    pub fn colored(self) -> String {
        let label = self.label();
        match self {
            Priority::None => label.to_string(),
            Priority::Urgent => label.red().bold().to_string(),
            Priority::High => label.yellow().bold().to_string(),
            Priority::Medium => label.blue().to_string(),
            Priority::Low => label.bright_black().to_string(),
        }
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl Serialize for Priority {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(self.as_i32())
    }
}

impl<'de> Deserialize<'de> for Priority {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = i32::deserialize(deserializer)?;
        Ok(Priority::from_i32(value))
    }
}
