#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::collections::HashMap;

use serde_aux::prelude::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};


// --------------------
// *    CONSTANTS     *
// --------------------

//#[derive(Serialize, Deserialize)]
pub enum ContentType { }

#[allow(dead_code)]
impl ContentType {
    //Make the fields just fields so they have integer values
    pub const PAGE: i8 = 1i8;
    pub const MODULE: i8 = 2i8;
    pub const FILE: i8 = 3i8;
    pub const USERPAGE: i8 = 4i8;
    pub const SYSTEM: i8 = 5i8;
}

pub enum UserType { }

#[allow(dead_code)]
impl UserType {
    pub const USER: i8 = 1i8;
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

#[derive(strum_macros::Display)]
#[allow(dead_code)]
pub enum SBSContentType
{
    forumcategory,
    forumthread
}


// -----------------------------
// *     RESULTS FROM API      *
// -----------------------------

#[derive(Serialize, Deserialize)]
pub struct About
{
    pub version: String,
    pub environment: String,
    pub runtime: String,
    pub contact: String
}

#[derive(Deserialize)]
pub struct SpecialCount
{
    pub specialCount: i32
}


// ----------------------------------
// *     VIEWS (READ AND WRITE)     *
// ----------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User
{
    pub id: i64,
    pub r#type: i8,
    pub username: String,
    pub avatar: String,
    pub special: Option<String>,
    //pub deleted: bool,
    #[serde(alias = "super", deserialize_with = "deserialize_bool_from_anything")]
    pub admin: bool,
    pub createDate : DateTime<Utc>
}

#[serde_with::skip_serializing_none] //MUST COME BEFORE
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[serde(default)]
pub struct Content //Remember, these are files, pages, threads etc. Lovely!
{
    //EVERYTHING is an option so we can construct these types JUST LIKE the web
    pub id: Option<i64>,
    pub name: Option<String>,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub deleted: bool, //WARN: this field may not represent reality!
    pub createUserId: Option<i64>,
    pub createDate : Option<DateTime<Utc>>,
    pub contentType : Option<i8>, // This is an enum, consider making values for this!
    pub parentId : Option<i64>,
    pub text: Option<String>, //This could be big?
    pub literalType: Option<String>,
    pub meta: Option<String>,
    pub description: Option<String>,
    pub hash: Option<String>,
    pub permissions: Option<HashMap<i64, String>>,
    pub values: Option<HashMap<String, serde_json::Value>>,
    pub keywords: Option<Vec<String>>,
    pub engagement: Option<HashMap<String, HashMap<String, i64>>>,
    pub lastCommentId: Option<i64>,
    pub commentCount: Option<i64>,
    pub watchCount: Option<i64>,
    pub keywordCount: Option<i64>,
    pub lastRevisionId: Option<i64>
}


#[serde_with::skip_serializing_none] //MUST COME BEFORE
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[serde(default)]
pub struct Message
{
    pub id: Option<i64>,
    pub contentId: Option<i64>,
    pub createUserId: Option<i64>,
    pub createDate : Option<DateTime<Utc>>,
    pub text: Option<String>,
    pub values: Option<HashMap<String, serde_json::Value>>,
    pub engagement: Option<HashMap<String, HashMap<String, i64>>>,
    pub editDate: Option<DateTime<Utc>>,
    pub editUserid: Option<i64>,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub edited: bool, 
    pub module: Option<String>
    //pub deleted: bool,
}


#[derive(Serialize, Deserialize)]
pub struct UserPrivate
{
    pub email: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestResult
{
    pub search: FullRequest,
    pub databaseTimes: HashMap<String, f64>,
    pub objects: HashMap<String, Vec<serde_json::Value>>,
    pub totalTime: f64,
    pub nonDbTime: f64,
    pub requestUser: Option<i64>
}


// -----------------------------
// *     QUERY PARAMETERS      *
// -----------------------------

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryImage
{
    pub size: Option<i64>,
    pub crop: Option<bool>
}


// ---------------------
// *     POST DATA     *
// ---------------------

#[derive(Serialize, Debug)]
pub struct Login
{
    pub username: String,
    pub password: String,
    pub expireSeconds: i64 
}

//Note: you WANT all these strings to be owned, even if it wastes memory or whatever,
//because you want to be able to construct and pass around whatever requests you want 
//from anywhere to anywhere and have the lifetimes of the internals strongly tied to the
//struct. It's about HOW you're using the struct, not simply "saving memory". 
#[serde_with::skip_serializing_none] //MUST COME BEFORE
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
        build_request!($type, $fields, Some($query), None, 0, 0, None)
    };
    ($type:expr, $fields:expr, $query:expr, $order:expr) => {
        build_request!($type, $fields, Some($query), Some($order), 0, 0, None)
    };
    ($type:expr, $fields:expr, $query:expr, $order:expr, $limit:expr) => {
        build_request!($type, $fields, Some($query), Some($order), $limit, 0, None)
    };
    ($type:expr, $fields:expr, $query:expr, $order:expr, $limit:expr, $skip:expr) => {
        build_request!($type, $fields, Some($query), Some($order), $limit, $skip, None)
    };
    ($type:expr, $fields:expr, $query:expr, $order:expr, $limit:expr, $skip:expr, $name:expr) => {
        crate::api_data::Request {
            name: $name,
            r#type: $type.to_string(), //Enum into string, 'strum_macros' helper
            fields: $fields,
            query: $query,
            order: $order,
            limit: $limit.into(),
            skip: $skip.into()
        }
    };
}
pub(crate) use build_request; // Now classic paths Just Work™

macro_rules! add_value {
    ($request:expr, $key:literal, $value:expr) => {
        $request.values.insert(String::from($key), $value.into());
    }
}
pub(crate) use add_value;

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


#[derive(Serialize, Deserialize, Debug)]
pub struct FileUploadAsObject {
    pub object: Content,
    pub base64blob: String, //This could be a VERY LARGE string!!!
}
