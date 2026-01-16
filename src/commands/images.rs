use std::path::{Path, PathBuf};

use regex::Regex;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use url::Url;

use crate::cli::DownloadImagesArgs;
use crate::client::LinearClient;
use crate::error::{LinearError, Result};
use crate::output;

/// Represents an image found in markdown content
#[derive(Debug, Clone)]
pub struct MarkdownImage {
    pub alt_text: String,
    pub url: String,
    pub index: usize,
}

/// Parse markdown content and extract all image URLs
/// Matches: ![alt text](url) and ![](url)
pub fn parse_markdown_images(markdown: &str) -> Vec<MarkdownImage> {
    let re = Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)").unwrap();

    re.captures_iter(markdown)
        .enumerate()
        .map(|(idx, cap)| MarkdownImage {
            alt_text: cap.get(1).map(|m| m.as_str().to_string()).unwrap_or_default(),
            url: cap.get(2).map(|m| m.as_str().to_string()).unwrap_or_default(),
            index: idx + 1, // 1-based indexing for user-facing
        })
        .collect()
}

/// Determine if a URL is a Linear-hosted asset (needs auth)
fn is_linear_url(url: &str) -> bool {
    url.contains("linear.app") || url.contains("uploads.linear.app")
}

/// Generate a filename for a downloaded image
fn generate_filename(issue_id: &str, image: &MarkdownImage, url: &Url) -> String {
    // Try to extract extension from URL path
    let extension = url
        .path_segments()
        .and_then(|mut segs| segs.next_back())
        .and_then(|filename| {
            let parts: Vec<&str> = filename.rsplitn(2, '.').collect();
            if parts.len() == 2 {
                Some(parts[0])
            } else {
                None
            }
        })
        .unwrap_or("png"); // Default to png if no extension found

    // Use alt text if meaningful, otherwise use index
    let name_part = if !image.alt_text.is_empty()
        && image.alt_text.len() < 50
        && image
            .alt_text
            .chars()
            .all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '_')
    {
        // Sanitize alt text for filename
        image
            .alt_text
            .replace(' ', "_")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
            .collect::<String>()
    } else {
        format!("image_{}", image.index)
    };

    format!("{}__{}.{}", issue_id, name_part, extension)
}

/// Download a single image
async fn download_image(
    http: &Client,
    api_key: &str,
    image: &MarkdownImage,
    output_dir: &Path,
    issue_id: &str,
) -> Result<PathBuf> {
    let url = Url::parse(&image.url).map_err(|_| LinearError::InvalidUrl(image.url.clone()))?;

    let mut request = http.get(url.clone());

    // Add auth header for Linear-hosted images
    if is_linear_url(&image.url) {
        request = request.header("Authorization", api_key);
    }

    let response = request.send().await?;

    if !response.status().is_success() {
        return Err(LinearError::ImageDownloadFailed {
            url: image.url.clone(),
            status: response.status().as_u16(),
        });
    }

    let bytes = response.bytes().await?;

    let filename = generate_filename(issue_id, image, &url);
    let file_path = output_dir.join(&filename);

    std::fs::write(&file_path, &bytes)?;

    Ok(file_path)
}

/// Result of a single image download attempt
#[derive(Debug)]
pub enum DownloadResult {
    Success {
        index: usize,
        path: PathBuf,
    },
    Failed {
        index: usize,
        url: String,
        error: String,
    },
}

impl DownloadResult {
    pub fn is_success(&self) -> bool {
        matches!(self, DownloadResult::Success { .. })
    }
}

/// Download all images (or specific index) from issue description
pub async fn download_images(
    api_key: &str,
    description: &str,
    issue_id: &str,
    output_dir: &Path,
    index: Option<usize>,
) -> Result<Vec<DownloadResult>> {
    // Ensure output directory exists
    if !output_dir.exists() {
        return Err(LinearError::OutputDirNotFound(output_dir.to_path_buf()));
    }

    let images = parse_markdown_images(description);
    let total = images.len();

    if images.is_empty() {
        return Ok(vec![]);
    }

    // Filter to specific index if provided
    let images_to_download: Vec<_> = if let Some(idx) = index {
        images.into_iter().filter(|img| img.index == idx).collect()
    } else {
        images
    };

    if images_to_download.is_empty() {
        if let Some(idx) = index {
            return Err(LinearError::ImageIndexOutOfBounds { index: idx, total });
        }
    }

    let http = Client::new();

    let mut results = Vec::new();

    for image in &images_to_download {
        let result = match download_image(&http, api_key, image, output_dir, issue_id).await {
            Ok(path) => DownloadResult::Success {
                index: image.index,
                path,
            },
            Err(e) => DownloadResult::Failed {
                index: image.index,
                url: image.url.clone(),
                error: e.to_string(),
            },
        };
        results.push(result);
    }

    Ok(results)
}

// GraphQL query for fetching issue
const GET_ISSUE_QUERY: &str = r#"
    query GetIssue($id: String!) {
        issue(id: $id) {
            id
            identifier
            description
        }
    }
"#;

#[derive(Deserialize)]
struct IssueResponse {
    issue: Option<IssueBasic>,
}

#[derive(Deserialize)]
struct IssueBasic {
    identifier: String,
    description: Option<String>,
}

/// Command handler: Download images from an issue's description
pub async fn download_images_command(client: &LinearClient, args: DownloadImagesArgs) -> Result<()> {
    // Create output directory if it doesn't exist
    if !args.output.exists() {
        std::fs::create_dir_all(&args.output)?;
    }

    let variables = json!({ "id": args.id });
    let response: IssueResponse = client.query(GET_ISSUE_QUERY, Some(variables)).await?;

    let issue = response
        .issue
        .ok_or_else(|| LinearError::IssueNotFound(args.id.clone()))?;

    let description = issue.description.as_deref().unwrap_or("");

    if description.is_empty() {
        output::print_message(&format!("Issue {} has no description", issue.identifier));
        return Ok(());
    }

    let results = download_images(
        client.api_key(),
        description,
        &issue.identifier,
        &args.output,
        args.index,
    )
    .await?;

    if results.is_empty() {
        output::print_message(&format!(
            "No images found in {} description",
            issue.identifier
        ));
        return Ok(());
    }

    print_download_results(&results);

    Ok(())
}

pub fn print_download_results(results: &[DownloadResult]) {
    let success_count = results.iter().filter(|r| r.is_success()).count();
    let fail_count = results.len() - success_count;

    for result in results {
        match result {
            DownloadResult::Success { index, path, .. } => {
                output::print_message(&format!("Downloaded image {} to {}", index, path.display()));
            }
            DownloadResult::Failed { index, url, error } => {
                eprintln!("Failed to download image {} ({}): {}", index, url, error);
            }
        }
    }

    if fail_count > 0 {
        output::print_message(&format!(
            "Downloaded {}/{} images ({} failed)",
            success_count,
            results.len(),
            fail_count
        ));
    } else if success_count > 1 {
        output::print_message(&format!("Downloaded {} images", success_count));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_markdown_images_empty() {
        let images = parse_markdown_images("");
        assert!(images.is_empty());
    }

    #[test]
    fn test_parse_markdown_images_no_images() {
        let images = parse_markdown_images("This is some text without images");
        assert!(images.is_empty());
    }

    #[test]
    fn test_parse_markdown_images_single() {
        let images = parse_markdown_images("![screenshot](https://example.com/img.png)");
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].alt_text, "screenshot");
        assert_eq!(images[0].url, "https://example.com/img.png");
        assert_eq!(images[0].index, 1);
    }

    #[test]
    fn test_parse_markdown_images_empty_alt() {
        let images = parse_markdown_images("![](https://example.com/img.png)");
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].alt_text, "");
        assert_eq!(images[0].url, "https://example.com/img.png");
    }

    #[test]
    fn test_parse_markdown_images_multiple() {
        let markdown = r#"
Some text
![first](https://example.com/1.png)
More text
![second](https://example.com/2.jpg)
"#;
        let images = parse_markdown_images(markdown);
        assert_eq!(images.len(), 2);
        assert_eq!(images[0].index, 1);
        assert_eq!(images[1].index, 2);
    }

    #[test]
    fn test_is_linear_url() {
        assert!(is_linear_url("https://uploads.linear.app/abc123"));
        assert!(is_linear_url("https://linear.app/uploads/img.png"));
        assert!(!is_linear_url("https://example.com/img.png"));
        assert!(!is_linear_url("https://imgur.com/abc.png"));
    }
}
