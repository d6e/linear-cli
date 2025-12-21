use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Attachment {
    pub id: String,
    pub title: String,
    pub url: Option<String>,
    pub subtitle: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Deserialize, Debug)]
pub struct UploadFile {
    #[serde(rename = "uploadUrl")]
    pub upload_url: String,
    #[serde(rename = "assetUrl")]
    pub asset_url: String,
    pub headers: Vec<UploadHeader>,
}

#[derive(Deserialize, Debug)]
pub struct UploadHeader {
    pub key: String,
    pub value: String,
}
