use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Attachment {
    pub id: String,
    pub title: String,
    pub url: Option<String>,
    pub subtitle: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}
