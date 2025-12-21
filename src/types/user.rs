use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
}
