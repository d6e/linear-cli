use std::fmt;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

/// Relation types between issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueRelationType {
    /// This issue blocks the related issue
    Blocks,
    /// This issue duplicates the related issue
    Duplicate,
    /// General relationship between issues
    Related,
}

impl fmt::Display for IssueRelationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Blocks => write!(f, "blocks"),
            Self::Duplicate => write!(f, "duplicate"),
            Self::Related => write!(f, "related"),
        }
    }
}

impl IssueRelationType {
    /// Get the inverse relation type label (for display from the other side).
    pub fn inverse_label(self) -> &'static str {
        match self {
            Self::Blocks => "blocked by",
            Self::Duplicate => "duplicate of",
            Self::Related => "related to",
        }
    }
}

/// A relation between two issues.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct IssueRelation {
    pub id: String,
    #[serde(rename = "type")]
    pub relation_type: IssueRelationType,
    pub issue: RelatedIssueRef,
    #[serde(rename = "relatedIssue")]
    pub related_issue: RelatedIssueRef,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

/// Minimal issue reference for relations.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RelatedIssueRef {
    pub id: String,
    pub identifier: String,
    pub title: String,
}
