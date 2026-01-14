use serde::{Deserialize, Serialize};
use serde_json::json;
use tabled::Tabled;

use crate::cli::RelateArgs;
use crate::client::LinearClient;
use crate::error::{LinearError, Result};
use crate::output::{self, truncate};
use crate::responses::Connection;
use crate::types::{IssueRelation, RelatedIssueRef};

const GET_ISSUE_RELATIONS_QUERY: &str = r#"
query GetIssueRelations($id: String!) {
    issue(id: $id) {
        id
        identifier
        relations {
            nodes {
                id
                type
                issue {
                    id
                    identifier
                    title
                }
                relatedIssue {
                    id
                    identifier
                    title
                }
            }
        }
        parent {
            id
            identifier
            title
        }
        children {
            nodes {
                id
                identifier
                title
            }
        }
    }
}
"#;

const CREATE_RELATION_MUTATION: &str = r#"
mutation CreateIssueRelation($input: IssueRelationCreateInput!) {
    issueRelationCreate(input: $input) {
        success
        issueRelation {
            id
            type
        }
    }
}
"#;

const DELETE_RELATION_MUTATION: &str = r#"
mutation DeleteIssueRelation($id: String!) {
    issueRelationDelete(id: $id) {
        success
    }
}
"#;

const UPDATE_ISSUE_PARENT_MUTATION: &str = r#"
mutation UpdateIssueParent($id: String!, $input: IssueUpdateInput!) {
    issueUpdate(id: $id, input: $input) {
        success
        issue {
            id
            identifier
        }
    }
}
"#;

const GET_ISSUE_ID_QUERY: &str = r#"
query GetIssueId($id: String!) {
    issue(id: $id) { id }
}
"#;

#[derive(Deserialize)]
struct IssueRelationsResponse {
    issue: Option<IssueWithRelations>,
}

#[derive(Deserialize)]
struct IssueWithRelations {
    identifier: String,
    relations: Connection<IssueRelation>,
    parent: Option<RelatedIssueRef>,
    children: Connection<RelatedIssueRef>,
}

#[derive(Deserialize)]
struct CreateRelationResponse {
    #[serde(rename = "issueRelationCreate")]
    issue_relation_create: RelationResult,
}

#[derive(Deserialize)]
struct RelationResult {
    success: bool,
}

#[derive(Deserialize)]
struct DeleteRelationResponse {
    #[serde(rename = "issueRelationDelete")]
    issue_relation_delete: RelationResult,
}

#[derive(Deserialize)]
struct UpdateIssueResponse {
    #[serde(rename = "issueUpdate")]
    issue_update: RelationResult,
}

#[derive(Deserialize)]
struct IssueIdResponse {
    issue: Option<IssueId>,
}

#[derive(Deserialize)]
struct IssueId {
    id: String,
}

#[derive(Tabled, Clone, Serialize)]
struct RelationRow {
    #[tabled(rename = "Type")]
    relation_type: String,
    #[tabled(rename = "Issue")]
    issue: String,
    #[tabled(rename = "Title")]
    title: String,
}

/// List all relations for an issue.
pub async fn list(client: &LinearClient, issue_id: &str) -> Result<()> {
    let variables = json!({ "id": issue_id });
    let response: IssueRelationsResponse = client
        .query(GET_ISSUE_RELATIONS_QUERY, Some(variables))
        .await?;

    let issue = response
        .issue
        .ok_or_else(|| LinearError::IssueNotFound(issue_id.to_string()))?;

    let mut rows: Vec<RelationRow> = Vec::new();

    // Add parent if exists
    if let Some(parent) = &issue.parent {
        rows.push(RelationRow {
            relation_type: "parent".to_string(),
            issue: parent.identifier.clone(),
            title: truncate(&parent.title, 50),
        });
    }

    // Add children
    for child in &issue.children.nodes {
        rows.push(RelationRow {
            relation_type: "child".to_string(),
            issue: child.identifier.clone(),
            title: truncate(&child.title, 50),
        });
    }

    // Add relations (normalizing direction)
    for rel in &issue.relations.nodes {
        let (rel_type, other) = if rel.issue.identifier == issue.identifier {
            // This issue is the source
            (rel.relation_type.to_string(), &rel.related_issue)
        } else {
            // This issue is the target (inverse relation)
            (rel.relation_type.inverse_label().to_string(), &rel.issue)
        };

        rows.push(RelationRow {
            relation_type: rel_type,
            issue: other.identifier.clone(),
            title: truncate(&other.title, 50),
        });
    }

    if rows.is_empty() {
        output::print_message(&format!("No relations for {}", issue.identifier));
        return Ok(());
    }

    output::print_table(
        &rows,
        |row| row.clone(),
        |row| format!("{}: {} - {}", row.relation_type, row.issue, row.title),
    );

    Ok(())
}

/// Create a relation between two issues.
pub async fn relate(client: &LinearClient, args: RelateArgs) -> Result<()> {
    // Resolve issue identifiers to UUIDs
    let source_id = resolve_issue_id(client, &args.source).await?;
    let target_id = resolve_issue_id(client, &args.target).await?;

    let variables = json!({
        "input": {
            "issueId": source_id,
            "relatedIssueId": target_id,
            "type": args.relation.to_string()
        }
    });

    let response: CreateRelationResponse = client
        .query(CREATE_RELATION_MUTATION, Some(variables))
        .await?;

    if response.issue_relation_create.success {
        output::print_message(&format!(
            "{} {} {}",
            args.source, args.relation, args.target
        ));
    }

    Ok(())
}

/// Remove a relation between two issues.
pub async fn unrelate(client: &LinearClient, source: &str, target: &str) -> Result<()> {
    // Find the relation between these two issues
    let variables = json!({ "id": source });
    let response: IssueRelationsResponse = client
        .query(GET_ISSUE_RELATIONS_QUERY, Some(variables))
        .await?;

    let issue = response
        .issue
        .ok_or_else(|| LinearError::IssueNotFound(source.to_string()))?;

    // Find matching relation
    let relation = issue.relations.nodes.iter().find(|r| {
        r.related_issue.identifier == target || r.issue.identifier == target
    });

    let relation = relation.ok_or_else(|| {
        LinearError::RelationNotFound(source.to_string(), target.to_string())
    })?;

    let delete_vars = json!({ "id": relation.id });
    let delete_response: DeleteRelationResponse = client
        .query(DELETE_RELATION_MUTATION, Some(delete_vars))
        .await?;

    if delete_response.issue_relation_delete.success {
        output::print_message(&format!("Removed relation between {} and {}", source, target));
    }

    Ok(())
}

/// Set the parent of an issue.
pub async fn set_parent(client: &LinearClient, id: &str, parent_id: &str) -> Result<()> {
    let resolved_parent_id = resolve_issue_id(client, parent_id).await?;

    let variables = json!({
        "id": id,
        "input": {
            "parentId": resolved_parent_id
        }
    });

    let response: UpdateIssueResponse = client
        .query(UPDATE_ISSUE_PARENT_MUTATION, Some(variables))
        .await?;

    if response.issue_update.success {
        output::print_message(&format!("Set {} as parent of {}", parent_id, id));
    }

    Ok(())
}

/// Remove the parent from an issue.
pub async fn remove_parent(client: &LinearClient, id: &str) -> Result<()> {
    let variables = json!({
        "id": id,
        "input": {
            "parentId": serde_json::Value::Null
        }
    });

    let response: UpdateIssueResponse = client
        .query(UPDATE_ISSUE_PARENT_MUTATION, Some(variables))
        .await?;

    if response.issue_update.success {
        output::print_message(&format!("Removed parent from {}", id));
    }

    Ok(())
}

/// Resolve an issue identifier (e.g., ENG-123) to its UUID.
async fn resolve_issue_id(client: &LinearClient, identifier: &str) -> Result<String> {
    // If it looks like a UUID already, return as-is
    if identifier.len() > 30 && !identifier.contains('-') {
        return Ok(identifier.to_string());
    }

    let response: IssueIdResponse = client
        .query(GET_ISSUE_ID_QUERY, Some(json!({ "id": identifier })))
        .await?;

    response
        .issue
        .map(|i| i.id)
        .ok_or_else(|| LinearError::IssueNotFound(identifier.to_string()))
}
