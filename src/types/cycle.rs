use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[allow(dead_code)]
pub struct Cycle {
    pub id: String,
    pub name: Option<String>,
    pub number: i32,
    #[serde(rename = "startsAt")]
    pub starts_at: String,
    #[serde(rename = "endsAt")]
    pub ends_at: String,
}
