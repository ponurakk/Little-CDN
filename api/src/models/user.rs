use serde::{ Deserialize, Serialize };

#[derive(Deserialize, Debug, Serialize)]
pub struct User {
    pub name: String,
    pub token: String,
}