use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LinearError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("API error (status {status}): {message}")]
    ApiError { status: u16, message: String },

    #[error("GraphQL errors: {}", messages.join(", "))]
    GraphQL { messages: Vec<String> },

    #[error("Empty response from API")]
    EmptyResponse,

    #[error("Failed to read config file at {path}: {source}")]
    ConfigRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse config file at {path}: {source}")]
    ConfigParse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("Could not determine config directory")]
    NoConfigDir,

    #[error(
        "No API key found. Set LINEAR_API_KEY env var or add api_key to ~/.config/linear/config.toml"
    )]
    MissingApiKey,

    #[error("Team not specified and no default_team in config")]
    NoTeam,

    #[error("Issue not found: {0}")]
    IssueNotFound(String),

    #[error("Cycle not found: {0}")]
    CycleNotFound(String),

    #[error("Team not found: {0}")]
    TeamNotFound(String),

    #[error("Workflow state not found: {0}")]
    WorkflowStateNotFound(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Failed to read file {path}: {source}")]
    FileRead {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("File upload failed (status {status}): {message}")]
    UploadFailed { status: u16, message: String },

    #[error("Image download failed for {url} (status {status})")]
    ImageDownloadFailed { url: String, status: u16 },

    #[error("Output directory not found: {0}")]
    OutputDirNotFound(PathBuf),

    #[error("Image index {index} out of bounds (issue has {total} images)")]
    ImageIndexOutOfBounds { index: usize, total: usize },

    #[error("Attachment index {index} out of bounds (issue has {total} attachments)")]
    AttachmentIndexOutOfBounds { index: usize, total: usize },

    #[error("Attachment download failed for {url} (status {status})")]
    AttachmentDownloadFailed { url: String, status: u16 },

    #[error("No attachments found for issue {0}")]
    NoAttachments(String),

    #[error("No relation found between {0} and {1}")]
    RelationNotFound(String, String),

    #[error("Label not found: {0}")]
    LabelNotFound(String),
}

pub type Result<T> = std::result::Result<T, LinearError>;
