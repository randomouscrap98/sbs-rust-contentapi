#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::collections::HashMap;

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

#[derive(Serialize, Deserialize, Clone)]
pub struct User
{
    pub id: i64,
    pub r#type: i8,
    pub username: String,
    pub avatar: String,
    pub special: Option<String>,
    //pub deleted: bool,
    #[serde(alias = "super")]
    pub admin: bool,
    pub createDate : DateTime<Utc>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Content //Remember, these are files, pages, threads etc. Lovely!
{
    pub id: i64,
    pub name: String,
    //pub deleted: i8, //bool, but api returns 0
    pub createUserId: i64,
    pub createDate : DateTime<Utc>,
    pub contentType : i8, // This is an enum, consider making values for this!
    pub parentId : i64,
    pub text: String, //This could be big?
    pub literalType: Option<String>,
    pub meta: Option<String>,
    pub description: Option<String>,
    pub hash: String,
    pub permissions: HashMap<i64, String>,
    pub values: HashMap<String, serde_json::Value>,
    pub keywords: Vec<String>,
    pub engagement: HashMap<String, HashMap<String, i64>>,
    pub lastCommentId: Option<i64>,
    pub commentCount: i64,
    pub watchCount: i64,
    pub keywordCount: i64,
    pub lastRevisionId: i64
}

impl Content {
    pub fn new(contentType: i8) -> Self {
        Content {
            id: 0,
            name: String::from(""),
            createUserId: 0,
            createDate : Utc::now(),
            contentType, //from params
            parentId: 0,
            text: String::from(""),
            literalType: None,
            meta: None,
            description: None,
            hash: String::from(""),
            permissions: HashMap::new(),
            values: HashMap::new(),
            keywords: Vec::new(),
            engagement: HashMap::new(),
            lastCommentId: None,
            commentCount: 0,
            watchCount: 0,
            keywordCount: 0,
            lastRevisionId: 0
        }
    }
}

//The content format will never change, so this "duplicate" is fine. This is
//used for the many queries that do NOT need everything
#[derive(Serialize, Deserialize, Debug)]
pub struct MinimalContent
{
    pub id: i64,
    pub name: String,
    //pub deleted: i8, //bool, but the api returns 0
    pub createUserId: i64,
    pub createDate : DateTime<Utc>,
    pub contentType : i8, // This is an enum, consider making values for this!
    pub parentId : i64,
    pub literalType: Option<String>,
    pub meta: Option<String>,
    pub description: Option<String>, // Is USUALLY small, and instantly retrievable
    pub hash: String,
    //These are expensive on QCS ONLY because off-topic takes a not-insignificant time to compute. but
    //they should be fast on SBS and they're just kinda necessary for too many 'minimal' things.
    pub lastCommentId: Option<i64>,
    pub commentCount: i64,
}

macro_rules! minimal_content {
    ($query:expr) => { 
        build_request!(
            RequestType::content, 
            String::from("id,name,deleted,createUserId,createDate,contentType,parentId,literalType,meta,description,hash,lastCommentId,commentCount"),
            $query
        )
    };
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
    pub requestUser: i64
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

#[derive(Serialize)]
pub struct Login
{
    pub username: String,
    pub password: String,
    pub expireSeconds: i64 
}

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
        build_request!($type, "*", None, None, 0, 0, None) 
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
            limit: $limit,
            skip: $skip
        }
    };
}

macro_rules! add_value {
    ($request:expr, $key:literal, $value:expr) => {
        $request.values.insert(String::from($key), $value.into());
    }
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


#[derive(Serialize, Deserialize, Debug)]
pub struct FileUploadAsObject {
    pub object: Content,
    pub base64blob: String, //This could be a VERY LARGE string!!!
}

pub(crate) use build_request; // Now classic paths Just Work™
pub(crate) use add_value;
pub(crate) use minimal_content;