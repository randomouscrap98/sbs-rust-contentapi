use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
pub struct About
{
    version: String,
    environment: String,
    runtime: String,
    contact: String
}

#[derive(Serialize, Deserialize)]
pub struct User
{
    id: u64,
    username: String,
    avatar: String
}


// These are query parameter data
#[derive(Serialize, Deserialize)]
pub struct QueryImage
{
    pub hash: String,
    pub size: Option<i64>,
    pub crop: Option<bool>
}

impl QueryImage {
    pub fn new(hash: &str) -> Self {
        Self {
            hash : String::from(hash),
            size: None,
            crop: None
        }
    }
}