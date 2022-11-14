use serde::{ Deserialize, Serialize };

#[derive(Deserialize, Serialize, Debug)]
pub struct File {
    pub original: String,
    pub changed: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Files {
    pub user: String,
    pub files: Vec<File>,
    pub files_count: u32,
}