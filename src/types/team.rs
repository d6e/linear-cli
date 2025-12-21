use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Team {
    pub id: String,
    pub key: String,
    pub name: String,
}
