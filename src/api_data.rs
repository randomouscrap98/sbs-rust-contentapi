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
#[derive(Serialize, Deserialize, Debug)]
pub struct QueryImage
{
    pub size: Option<i64>,
    pub crop: Option<bool>
}
