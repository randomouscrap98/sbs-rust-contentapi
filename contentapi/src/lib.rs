#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::collections::HashMap;

use serde_aux::prelude::*; //Necessary to deserialize bool from "anything"
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub mod endpoints;
pub mod forms;
pub mod conversion;


macro_rules! enum_type {
    ($name:ident => {
        $($item:ident,)*
    }) => {
        #[allow(dead_code)] //man, idk if i'll use ALL of them but I WANT them
        pub enum $name {
            $($item,)*
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                $(
                    Self::$item => { write!(f, stringify!($item)) },
                )*
                }
            }
        }
    };
}

// --------------------
// *    CONSTANTS     *
// --------------------

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

enum_type!{
    RequestType => {
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
        message_engagement,
    }
}

enum_type!{
    SBSContentType => {
        forumcategory,
        forumthread,
    }
}


// -----------------------------
// *     RESULTS FROM API      *
// -----------------------------

#[derive(Serialize, Deserialize, Debug)]
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

//#[serde(skip_serializing_if = "Option::is_none")]
//#[serde_with::skip_serializing_none] //MUST COME BEFORE
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[serde(default)]
pub struct Content //Remember, these are files, pages, threads etc. Lovely!
{
    //EVERYTHING is an option so we can construct these types JUST LIKE the web
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub deleted: bool, //WARN: this field may not represent reality!
    #[serde(skip_serializing_if = "Option::is_none")]
    pub createUserId: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub createDate : Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contentType : Option<i8>, // This is an enum, consider making values for this!
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parentId : Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>, //This could be big?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub literalType: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<HashMap<String, String>>, //JSON doesn't have int keys right? You'll just have to parse it
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engagement: Option<HashMap<String, HashMap<String, i64>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lastCommentId: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commentCount: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watchCount: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywordCount: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lastRevisionId: Option<i64>
}

macro_rules! make_values {
    ($($field_name:literal : $field_value:expr),*$(,)*) => {
        vec![$((String::from($field_name), serde_json::to_value($field_value)?))*]
            .into_iter().collect::<std::collections::HashMap<String, serde_json::Value>>()
    };
}
pub(crate) use make_values;

macro_rules! add_value {
    ($request:expr, $key:literal, $value:expr) => {
        $request.values.insert(String::from($key), $value.into());
    }
}
pub(crate) use add_value;


//#[serde_with::skip_serializing_none] //MUST COME BEFORE
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[serde(default)]
pub struct Message
{
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contentId: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub createUserId: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub createDate : Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engagement: Option<HashMap<String, HashMap<String, i64>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editDate: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editUserId: Option<i64>,
    //#[serde(deserialize_with = "deserialize_bool_from_anything")]
    //pub edited: bool, 
    #[serde(skip_serializing_if = "Option::is_none")]
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

//Note: you WANT all these strings to be owned, even if it wastes memory or whatever,
//because you want to be able to construct and pass around whatever requests you want 
//from anywhere to anywhere and have the lifetimes of the internals strongly tied to the
//struct. It's about HOW you're using the struct, not simply "saving memory". 
//#[serde_with::skip_serializing_none] //MUST COME BEFORE
#[derive(Serialize, Deserialize, Debug)]
pub struct Request
{
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub r#type: String,
    pub fields: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>, 
    #[serde(skip_serializing_if = "Option::is_none")]
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
            r#type: $type.to_string(), //enum to string, because we implement display on all
            fields: $fields,
            query: $query,
            order: $order,
            limit: $limit.into(),
            skip: $skip.into()
        }
    };
}
pub(crate) use build_request; // Now classic paths Just Work™

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
