use serde::{Deserialize, Serialize};

use super::{Cycle, Project, Team, User};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[allow(dead_code)]
pub struct Issue {
    pub id: String,
    pub identifier: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: i32,
    pub state: Option<WorkflowState>,
    pub assignee: Option<User>,
    pub team: Team,
    pub project: Option<Project>,
    pub cycle: Option<Cycle>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[allow(dead_code)]
pub struct WorkflowState {
    pub id: String,
    pub name: String,
    pub color: String,
}
