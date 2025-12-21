use serde::{Deserialize, Serialize};
use serde_json::json;
use tabled::Tabled;

use crate::cli::CommentArgs;
use crate::client::LinearClient;
use crate::error::{LinearError, Result};
use crate::output::{self, format_relative, truncate};

const LIST_COMMENTS_QUERY: &str = r#"
query ListComments($issueId: String!) {
    issue(id: $issueId) {
        comments {
            nodes {
                id
                body
                createdAt
                user {
                    id
                    name
                }
            }
        }
    }
}
"#;

const CREATE_COMMENT_MUTATION: &str = r#"
mutation CreateComment($issueId: String!, $body: String!) {
    commentCreate(input: { issueId: $issueId, body: $body }) {
        success
        comment {
            id
            body
        }
    }
}
"#;

#[derive(Deserialize)]
struct CommentsResponse {
    issue: Option<IssueWithComments>,
}

#[derive(Deserialize)]
struct IssueWithComments {
    comments: CommentsConnection,
}

#[derive(Deserialize)]
struct CommentsConnection {
    nodes: Vec<Comment>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[allow(dead_code)]
pub struct Comment {
    pub id: String,
    pub body: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    pub user: Option<CommentUser>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[allow(dead_code)]
pub struct CommentUser {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize)]
struct CreateCommentResponse {
    #[serde(rename = "commentCreate")]
    comment_create: CommentCreateResult,
}

#[derive(Deserialize)]
struct CommentCreateResult {
    success: bool,
    #[allow(dead_code)]
    comment: Option<CreatedComment>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct CreatedComment {
    id: String,
    body: String,
}

#[derive(Tabled)]
struct CommentRow {
    #[tabled(rename = "Author")]
    author: String,
    #[tabled(rename = "Comment")]
    body: String,
    #[tabled(rename = "When")]
    created_at: String,
}

impl From<&Comment> for CommentRow {
    fn from(comment: &Comment) -> Self {
        Self {
            author: comment
                .user
                .as_ref()
                .map(|u| u.name.clone())
                .unwrap_or_else(|| "Unknown".to_string()),
            body: truncate(&comment.body.replace('\n', " "), 60),
            created_at: format_relative(&comment.created_at),
        }
    }
}

pub async fn list(client: &LinearClient, issue_id: &str) -> Result<()> {
    let variables = json!({ "issueId": issue_id });
    let response: CommentsResponse = client.query(LIST_COMMENTS_QUERY, Some(variables)).await?;

    let comments = response
        .issue
        .ok_or_else(|| LinearError::IssueNotFound(issue_id.to_string()))?
        .comments
        .nodes;

    if comments.is_empty() {
        output::print_message(&format!("No comments on {issue_id}"));
        return Ok(());
    }

    output::print_table(&comments, |c| CommentRow::from(c));

    Ok(())
}

pub async fn add(client: &LinearClient, args: CommentArgs) -> Result<()> {
    let variables = json!({
        "issueId": args.id,
        "body": args.body
    });

    let response: CreateCommentResponse = client
        .query(CREATE_COMMENT_MUTATION, Some(variables))
        .await?;

    if response.comment_create.success {
        output::print_message(&format!("Added comment to {}", args.id));
    }

    Ok(())
}
