#![allow(non_snake_case)]

use chrono::{DateTime, Utc};
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
    avatar: String,
    createDate : DateTime<Utc>
}

#[derive(Serialize, Deserialize)]
pub struct UserPrivate
{
    email: String
}


// These are query parameter data
#[derive(Serialize, Deserialize, Debug)]
pub struct QueryImage
{
    pub size: Option<i64>,
    pub crop: Option<bool>
}


// Data to submit to the api

#[derive(Serialize)]
pub struct Login
{
    pub username: String,
    pub password: String,
    pub expireSeconds: i64 
}