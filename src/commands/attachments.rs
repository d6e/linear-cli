use std::path::Path;

use serde::Deserialize;
use serde_json::json;
use tabled::Tabled;

use crate::cli::{AttachUrlArgs, UploadFileArgs};
use crate::client::LinearClient;
use crate::error::{LinearError, Result};
use crate::output::{self, format_date_only, truncate};
use crate::types::Attachment;

const LIST_ATTACHMENTS_QUERY: &str = r#"
query ListAttachments($issueId: String!) {
    issue(id: $issueId) {
        attachments {
            nodes {
                id
                title
                url
                subtitle
                createdAt
            }
        }
    }
}
"#;

const ATTACH_URL_MUTATION: &str = r#"
mutation AttachmentLinkURL($issueId: String!, $url: String!, $title: String) {
    attachmentLinkURL(issueId: $issueId, url: $url, title: $title) {
        success
        attachment {
            id
            title
            url
        }
    }
}
"#;

const FILE_UPLOAD_MUTATION: &str = r#"
mutation FileUpload($filename: String!, $contentType: String!, $size: Int!) {
    fileUpload(filename: $filename, contentType: $contentType, size: $size) {
        uploadFile {
            uploadUrl
            assetUrl
            headers {
                key
                value
            }
        }
    }
}
"#;

const CREATE_ATTACHMENT_MUTATION: &str = r#"
mutation AttachmentCreate($issueId: String!, $url: String!, $title: String!) {
    attachmentCreate(input: { issueId: $issueId, url: $url, title: $title }) {
        success
        attachment {
            id
            title
            url
        }
    }
}
"#;

#[derive(Deserialize)]
struct AttachmentsResponse {
    issue: Option<IssueWithAttachments>,
}

#[derive(Deserialize)]
struct IssueWithAttachments {
    attachments: AttachmentsConnection,
}

#[derive(Deserialize)]
struct AttachmentsConnection {
    nodes: Vec<Attachment>,
}

#[derive(Deserialize)]
struct AttachUrlResponse {
    #[serde(rename = "attachmentLinkURL")]
    attachment_link_url: AttachmentResult,
}

#[derive(Deserialize)]
struct AttachmentResult {
    success: bool,
    attachment: Option<AttachmentInfo>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct AttachmentInfo {
    title: String,
    url: Option<String>,
}

#[derive(Deserialize)]
struct FileUploadResponse {
    #[serde(rename = "fileUpload")]
    file_upload: FileUploadResult,
}

#[derive(Deserialize)]
struct FileUploadResult {
    #[serde(rename = "uploadFile")]
    upload_file: UploadFileInfo,
}

#[derive(Deserialize)]
struct UploadFileInfo {
    #[serde(rename = "uploadUrl")]
    upload_url: String,
    #[serde(rename = "assetUrl")]
    asset_url: String,
    headers: Vec<UploadHeader>,
}

#[derive(Deserialize)]
struct UploadHeader {
    key: String,
    value: String,
}

#[derive(Deserialize)]
struct CreateAttachmentResponse {
    #[serde(rename = "attachmentCreate")]
    attachment_create: AttachmentResult,
}

#[derive(Tabled)]
struct AttachmentRow {
    #[tabled(rename = "Title")]
    title: String,
    #[tabled(rename = "URL")]
    url: String,
    #[tabled(rename = "Created")]
    created_at: String,
}

impl From<&Attachment> for AttachmentRow {
    fn from(attachment: &Attachment) -> Self {
        Self {
            title: truncate(&attachment.title, 40),
            url: truncate(attachment.url.as_deref().unwrap_or("-"), 50),
            created_at: format_date_only(&attachment.created_at),
        }
    }
}

pub async fn list(client: &LinearClient, issue_id: &str) -> Result<()> {
    let variables = json!({ "issueId": issue_id });
    let response: AttachmentsResponse = client.query(LIST_ATTACHMENTS_QUERY, Some(variables)).await?;

    let attachments = response
        .issue
        .ok_or_else(|| LinearError::IssueNotFound(issue_id.to_string()))?
        .attachments
        .nodes;

    if attachments.is_empty() {
        output::print_message(&format!("No attachments found for {issue_id}"));
        return Ok(());
    }

    output::print_table(&attachments, |a| AttachmentRow::from(a));

    Ok(())
}

pub async fn attach_url(client: &LinearClient, args: AttachUrlArgs) -> Result<()> {
    let title = args.title.unwrap_or_else(|| args.url.clone());

    let variables = json!({
        "issueId": args.id,
        "url": args.url,
        "title": title
    });

    let response: AttachUrlResponse = client.query(ATTACH_URL_MUTATION, Some(variables)).await?;

    if response.attachment_link_url.success {
        if let Some(attachment) = response.attachment_link_url.attachment {
            output::print_message(&format!("Attached \"{}\" to {}", attachment.title, args.id));
        }
    }

    Ok(())
}

pub async fn upload_file(client: &LinearClient, args: UploadFileArgs) -> Result<()> {
    let path = Path::new(&args.file);

    if !path.exists() {
        return Err(LinearError::FileNotFound(args.file.clone()));
    }

    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file")
        .to_string();

    let title = args.title.unwrap_or_else(|| filename.clone());

    // Read file
    let file_data = std::fs::read(path).map_err(|e| LinearError::FileRead {
        path: args.file.clone(),
        source: e,
    })?;

    let file_size = file_data.len() as i32;

    // Guess content type from extension
    let content_type = guess_content_type(&filename);

    // Step 1: Get upload URL
    let variables = json!({
        "filename": filename,
        "contentType": content_type,
        "size": file_size
    });

    let upload_response: FileUploadResponse = client
        .query(FILE_UPLOAD_MUTATION, Some(variables))
        .await?;

    let upload_info = upload_response.file_upload.upload_file;

    // Step 2: Upload file to the signed URL
    let http_client = reqwest::Client::new();
    let mut request = http_client
        .put(&upload_info.upload_url)
        .body(file_data)
        .header("Content-Type", &content_type);

    // Add required headers from the mutation response
    for header in &upload_info.headers {
        request = request.header(&header.key, &header.value);
    }

    let upload_result = request.send().await?;

    if !upload_result.status().is_success() {
        return Err(LinearError::UploadFailed {
            status: upload_result.status().as_u16(),
            message: upload_result.text().await.unwrap_or_default(),
        });
    }

    // Step 3: Create attachment linking to the uploaded file
    let attach_variables = json!({
        "issueId": args.id,
        "url": upload_info.asset_url,
        "title": title
    });

    let attach_response: CreateAttachmentResponse = client
        .query(CREATE_ATTACHMENT_MUTATION, Some(attach_variables))
        .await?;

    if attach_response.attachment_create.success {
        output::print_message(&format!("Uploaded \"{}\" to {}", title, args.id));
    }

    Ok(())
}

fn guess_content_type(filename: &str) -> String {
    let ext = filename
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "pdf" => "application/pdf",
        "txt" => "text/plain",
        "md" => "text/markdown",
        "json" => "application/json",
        "xml" => "application/xml",
        "zip" => "application/zip",
        "tar" => "application/x-tar",
        "gz" => "application/gzip",
        _ => "application/octet-stream",
    }
    .to_string()
}
