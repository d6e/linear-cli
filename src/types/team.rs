use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Team {
    pub id: String,
    pub key: String,
    pub name: String,
}
