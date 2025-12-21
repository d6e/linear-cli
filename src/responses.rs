//! Shared GraphQL response types used across commands.

use serde::Deserialize;

/// Pagination info for cursor-based pagination.
#[derive(Deserialize)]
pub struct PageInfo {
    #[serde(rename = "hasNextPage")]
    pub has_next_page: bool,
    #[serde(rename = "endCursor")]
    pub end_cursor: Option<String>,
}

/// Viewer (current user) response.
#[derive(Deserialize)]
pub struct ViewerResponse {
    pub viewer: Viewer,
}

#[derive(Deserialize)]
pub struct Viewer {
    pub id: String,
}

/// Workflow state for status lookups.
#[derive(Deserialize)]
pub struct WorkflowStateNode {
    pub id: String,
    pub name: String,
}

/// Team node with minimal fields for ID lookups.
#[derive(Deserialize)]
pub struct TeamNode {
    pub id: String,
}

/// Minimal issue info returned after create/update.
#[derive(Deserialize)]
pub struct CreatedIssue {
    pub identifier: String,
    pub title: String,
}
