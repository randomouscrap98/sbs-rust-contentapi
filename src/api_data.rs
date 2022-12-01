#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

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

#[derive(strum_macros::Display)]
#[allow(dead_code)] //man, idk if i'll use ALL of them but I WANT them
pub enum RequestType
{
    user,
    content,
    message,
    activity,
    watch,
    adminlog,
    uservariable,
    message_aggregate,
    activity_aggregate,
    content_engagement,
    ban,
    keyword_aggregate,
    message_engagement
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Request
{
    pub name: Option<String>,
    pub r#type: String,
    pub fields: String,
    pub query: Option<String>, 
    pub order: Option<String>,
    pub limit: i64, //Everything is i64 so it's easier to serialize/deserialize
    pub skip: i64
}

macro_rules! build_request {
    //All these expect the RequestType enum
    ($type:expr) => { 
        build_request!($type, String::from("*"), None, None, 0, 0, None) 
    };
    ($type:expr, $fields:expr) => { 
        build_request!($type, $fields, None, None, 0, 0, None) 
    };
    ($type:expr, $fields:expr, $query:expr) => {
        build_request!($type, $fields, $query, None, 0, 0, None)
    };
    ($type:expr, $fields:expr, $query:expr, $order:expr) => {
        build_request!($type, $fields, $query, $order, 0, 0, None)
    };
    ($type:expr, $fields:expr, $query:expr, $order:expr, $limit:expr) => {
        build_request!($type, $fields, $query, $order, $limit, 0, None)
    };
    ($type:expr, $fields:expr, $query:expr, $order:expr, $limit:expr, $skip:expr) => {
        build_request!($type, $fields, $query, $order, $limit, $skip, None)
    };
    ($type:expr, $fields:expr, $query:expr, $order:expr, $limit:expr, $skip:expr, $name:expr) => {
        api_data::Request {
            name: $name,
            r#type: $type.to_string(), //Enum into string, 'strum_macros' helper
            fields: $fields,
            query: $query,
            order: $order,
            limit: $limit,
            skip: $skip
        }
    };
}
//impl Request {
//    pub fn 
//}

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

pub(crate) use build_request; // Now classic paths Just Work™
