use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub state: Option<String>,
}
