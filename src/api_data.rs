#![allow(non_snake_case)]

use std::collections::HashMap;

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

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestResult
{
    pub search: FullRequest,
    pub databaseTimes: HashMap<String, i64>,
    pub objects: HashMap<String, serde_json::Value>,
    pub totalTime: i64,
    pub nonDbTime: i64,
    pub requestUser: i64
}


// Data to submit to the api

#[derive(Serialize)]
pub struct Login
{
    pub username: String,
    pub password: String,
    pub expireSeconds: i64 
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Request
{
    pub name: String,
    pub r#type: String,
    pub fields: String,
    pub query: String, 
    pub order: String,
    pub limit: i64, //Everything is i64 so it's easier to serialize/deserialize
    pub skip: i64
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FullRequest
{
    pub values: HashMap<String, serde_json::Value>, //HashMap<String, Box<Serialize>>,
    pub requests: Vec::<Request>
}

impl FullRequest {
    pub fn new() -> Self {
        FullRequest { values: HashMap::new(), requests: Vec::new() }
    }
}