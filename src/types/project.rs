use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub state: Option<String>,
}
