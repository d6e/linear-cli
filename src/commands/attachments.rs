use std::path::{Path, PathBuf};

use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use tabled::Tabled;
use url::Url;

use crate::cli::{AttachUrlArgs, DownloadAttachmentsArgs, UploadFileArgs};
use crate::client::LinearClient;
use crate::error::{LinearError, Result};
use crate::output::{self, format_date_only, truncate};
use crate::responses::Connection;
use crate::types::Attachment;

#[derive(Tabled)]
struct AttachmentRow {
    #[tabled(rename = "Title")]
    title: String,
    #[tabled(rename = "URL")]
    url: String,
    #[tabled(rename = "Created")]
    created: String,
}

impl From<&Attachment> for AttachmentRow {
    fn from(attachment: &Attachment) -> Self {
        Self {
            title: truncate(&attachment.title, 40),
            url: truncate(attachment.url.as_deref().unwrap_or("-"), 50),
            created: format_date_only(&attachment.created_at),
        }
    }
}

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
    attachments: Connection<Attachment>,
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
struct AttachmentInfo {
    title: String,
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

pub async fn list(client: &LinearClient, issue_id: &str) -> Result<()> {
    let variables = json!({ "issueId": issue_id });
    let response: AttachmentsResponse = client
        .query(LIST_ATTACHMENTS_QUERY, Some(variables))
        .await?;

    let attachments = response
        .issue
        .ok_or_else(|| LinearError::IssueNotFound(issue_id.to_string()))?
        .attachments
        .nodes;

    if attachments.is_empty() {
        output::print_message(&format!("No attachments found for {issue_id}"));
        return Ok(());
    }

    output::print_table(
        &attachments,
        |attachment| AttachmentRow::from(attachment),
        |attachment| {
            format!(
                "{} | {}",
                truncate(&attachment.title, 40),
                attachment.url.as_deref().unwrap_or("-")
            )
        },
    );

    Ok(())
}

pub async fn attach_url(client: &LinearClient, args: AttachUrlArgs) -> Result<()> {
    // Validate URL format
    Url::parse(&args.url).map_err(|_| LinearError::InvalidUrl(args.url.clone()))?;

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

    let upload_response: FileUploadResponse =
        client.query(FILE_UPLOAD_MUTATION, Some(variables)).await?;

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
            message: upload_result
                .text()
                .await
                .unwrap_or_else(|_| "<failed to read response body>".to_string()),
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
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();

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

/// Determine if a URL is a Linear-hosted asset (needs auth)
fn is_linear_url(url: &str) -> bool {
    url.contains("linear.app") || url.contains("uploads.linear.app")
}

/// Generate a filename for a downloaded attachment
fn generate_attachment_filename(issue_id: &str, attachment: &Attachment, index: usize) -> String {
    let title = &attachment.title;

    // Try to extract extension from URL if present
    let extension = attachment
        .url
        .as_ref()
        .and_then(|url_str| Url::parse(url_str).ok())
        .and_then(|url| {
            url.path_segments()
                .and_then(|mut segs| segs.next_back())
                .and_then(|filename| {
                    let parts: Vec<&str> = filename.rsplitn(2, '.').collect();
                    if parts.len() == 2 {
                        Some(parts[0].to_string())
                    } else {
                        None
                    }
                })
        });

    // Check if title already has an extension
    let title_has_ext = title
        .rsplit('.')
        .next()
        .map(|ext| ext.len() <= 5 && ext.chars().all(|c| c.is_alphanumeric()))
        .unwrap_or(false);

    // Sanitize title for use as filename
    let sanitized_title: String = title
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    if title_has_ext {
        format!("{}__{}", issue_id, sanitized_title)
    } else if let Some(ext) = extension {
        format!("{}__{}_{}.{}", issue_id, sanitized_title, index, ext)
    } else {
        format!("{}__{}_{}", issue_id, sanitized_title, index)
    }
}

/// Download a single attachment
async fn download_attachment(
    http: &Client,
    api_key: &str,
    attachment: &Attachment,
    output_dir: &Path,
    issue_id: &str,
    index: usize,
) -> Result<PathBuf> {
    let url_str = attachment
        .url
        .as_ref()
        .ok_or_else(|| LinearError::AttachmentDownloadFailed {
            url: "(no url)".to_string(),
            status: 0,
        })?;

    let url = Url::parse(url_str).map_err(|_| LinearError::InvalidUrl(url_str.clone()))?;

    let mut request = http.get(url.clone());

    // Add auth header for Linear-hosted attachments
    if is_linear_url(url_str) {
        request = request.header("Authorization", api_key);
    }

    let response = request.send().await?;

    if !response.status().is_success() {
        return Err(LinearError::AttachmentDownloadFailed {
            url: url_str.clone(),
            status: response.status().as_u16(),
        });
    }

    let bytes = response.bytes().await?;

    let filename = generate_attachment_filename(issue_id, attachment, index);
    let file_path = output_dir.join(&filename);

    std::fs::write(&file_path, &bytes)?;

    Ok(file_path)
}

pub async fn download(client: &LinearClient, args: DownloadAttachmentsArgs) -> Result<()> {
    // Create output directory if it doesn't exist
    if !args.output.exists() {
        std::fs::create_dir_all(&args.output)?;
    }

    // Fetch attachments
    let variables = json!({ "issueId": args.id });
    let response: AttachmentsResponse = client
        .query(LIST_ATTACHMENTS_QUERY, Some(variables))
        .await?;

    let attachments = response
        .issue
        .ok_or_else(|| LinearError::IssueNotFound(args.id.clone()))?
        .attachments
        .nodes;

    if attachments.is_empty() {
        return Err(LinearError::NoAttachments(args.id.clone()));
    }

    let total = attachments.len();

    // Filter to specific index if provided (1-based indexing)
    let attachments_to_download: Vec<_> = if let Some(idx) = args.index {
        if idx == 0 || idx > total {
            return Err(LinearError::AttachmentIndexOutOfBounds { index: idx, total });
        }
        vec![(idx, &attachments[idx - 1])]
    } else {
        attachments
            .iter()
            .enumerate()
            .map(|(i, a)| (i + 1, a))
            .collect()
    };

    let http = Client::new();
    let api_key = client.api_key();

    for (index, attachment) in attachments_to_download {
        match download_attachment(&http, api_key, attachment, &args.output, &args.id, index).await {
            Ok(path) => {
                output::print_message(&format!(
                    "Downloaded attachment {} to {}",
                    index,
                    path.display()
                ));
            }
            Err(e) => {
                eprintln!(
                    "Failed to download attachment {} ({}): {}",
                    index, attachment.title, e
                );
            }
        }
    }

    Ok(())
}

/// Download all attachments to a directory (returns count of successful downloads)
pub async fn download_to_dir(
    client: &LinearClient,
    issue_id: &str,
    output_dir: &Path,
) -> Result<usize> {
    // Fetch attachments
    let variables = json!({ "issueId": issue_id });
    let response: AttachmentsResponse = client
        .query(LIST_ATTACHMENTS_QUERY, Some(variables))
        .await?;

    let attachments = match response.issue {
        Some(issue) => issue.attachments.nodes,
        None => return Ok(0),
    };

    if attachments.is_empty() {
        return Ok(0);
    }

    let http = Client::new();
    let api_key = client.api_key();
    let mut success_count = 0;

    for (index, attachment) in attachments.iter().enumerate() {
        if download_attachment(&http, api_key, attachment, output_dir, issue_id, index + 1)
            .await
            .is_ok()
        {
            success_count += 1;
        }
    }

    Ok(success_count)
}
