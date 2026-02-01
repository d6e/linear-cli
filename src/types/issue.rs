use serde::{Deserialize, Serialize};

use super::{Cycle, Priority, Project, Team, User};
use crate::commands::labels::Label;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Issue {
    pub id: String,
    pub identifier: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: Priority,
    pub state: Option<WorkflowState>,
    pub assignee: Option<User>,
    pub team: Team,
    pub project: Option<Project>,
    pub cycle: Option<Cycle>,
    pub labels: Option<LabelNodes>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LabelNodes {
    pub nodes: Vec<Label>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct WorkflowState {
    pub id: String,
    pub name: String,
    pub color: String,
}
