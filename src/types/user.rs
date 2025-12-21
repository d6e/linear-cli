use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[allow(dead_code)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
}
