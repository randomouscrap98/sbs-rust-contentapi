#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::collections::HashMap;

use serde_aux::prelude::*; //Necessary to deserialize bool from "anything"
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod endpoints;
pub mod forms;
pub mod conversion;
pub mod search;
pub mod permissions;

/// Create the values for ['Content'] or ['Message'] from a simple list of key : value pairs
#[macro_export]
macro_rules! make_values {
    ($($field_name:literal : $field_value:expr),*$(,)?) => {
        vec![$((String::from($field_name), serde_json::to_value($field_value)?))*]
            .into_iter().collect::<std::collections::HashMap<String, serde_json::Value>>()
    };
}

#[macro_export]
macro_rules! make_permissions {
    ($($field_name:literal : $field_value:expr),*$(,)?) => {
        vec![$((String::from($field_name), String::from($field_value)))*]
            .into_iter().collect::<std::collections::HashMap<String, String>>()
    };
}

/// Add a single value to an existing values hash from [`Content`] or [`Message`], 
/// or a value to a request (they amount to the same thing)
#[macro_export]
macro_rules! add_value {
    ($request:expr, $key:literal, $value:expr) => {
        $request.values.insert(String::from($key), $value.into());
    }
}

/// Create a string enum, used for various human readable types from the API
#[macro_export]
macro_rules! string_enum {
    ($name:ident => {
        $($item:ident),*$(,)?
    }) => {
        #[allow(dead_code)] //man, idk if i'll use ALL of them but I WANT them
        pub enum $name {
            $($item,)*
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", self.to_literal())
            }
        }

        /// Convert enum to string literal; prefer this function over display
        impl $name {
            fn to_literal(&self) -> &'static str {
                match self {
                $(
                    Self::$item => stringify!($item),
                )*
                }
            }
        }
    };
}

/// Create a in "integer" enum, used for codified values from the API
#[macro_export]
macro_rules! byte_enum {
    ($name:ident => {
        $(($item:ident : $val:literal)),*$(,)?
    }) => {
        #[allow(dead_code)] //man, idk if i'll use ALL of them but I WANT them
        pub enum $name { }

        impl $name {
        $(
            pub const $item: i8 = $val;
        )*
        }
    };
}

// --------------------
// *    CONSTANTS     *
// --------------------

byte_enum!{ ContentType => {
    (PAGE:1i8),
    (MODULE:2i8),
    (FILE:3i8),
    (USERPAGE:4i8),
    (SYSTEM:5i8)
}}

byte_enum!{ UserType => {
    (USER:1i8),
    (GROUP:2i8)
}}

byte_enum!{ BanType => {
    (NONE:0i8),
    (PUBLIC:1i8),
    (PRIVATE:2i8)
}}

byte_enum!{ UserRelationType => {
    (INGROUP:1i8),
    (ASSIGNCONTENT:2i8)
}}

byte_enum!{ UserAction => {
    (CREATE:1i8),
    (READ:2i8),
    (UPDATE:4i8),
    (DELETE:8i8)
}}

string_enum!{ RequestType => {
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
    userrelation
}}

/*string_enum!{ SBSContentType => {
    forumcategory,
    forumthread,
    submissions,
    program,
    resource,
    directmessage,
    directmessages,
    alert,
    frontpage
}}*/


// -----------------------------
// *     RESULTS FROM API      *
// -----------------------------

#[derive(Deserialize, Debug)]
pub struct About
{
    pub version: String,
    pub environment: String,
    pub runtime: String,
    pub contact: String
}

#[derive(Deserialize, Debug)]
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
    pub createDate : DateTime<Utc>,
    pub groups: Vec<i64>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserBan
{
    pub id: i64,
    pub createDate: DateTime<Utc>,
    pub expireDate: DateTime<Utc>,
    pub createUserId: i64,
    pub bannedUserId: i64,
    pub message: Option<String>,
    pub r#type: i8
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

impl Content {
    pub fn get_value_str(&self, key: &str) -> Option<&str>
    {
        if let Some(ref values) = self.values {
            values.get(key).and_then(|v| v.as_str())
        } else { None }
    }
    pub fn get_value_string(&self, key: &str) -> Option<String>
    {
        self.get_value_str(key).map(|v| v.to_string())
    }
    pub fn get_value_array(&self, key: &str) -> Option<&Vec<Value>>
    {
        if let Some(ref values) = self.values {
            values.get(key).and_then(|v| v.as_array())
        } else { None }
    }
}


#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[serde(default)]
pub struct ContentEngagement
{
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id : Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub userId: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type : Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engagement : Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub createDate : Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contentId: Option<i64>,
}


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
    pub module: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_literalType: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_contentType: Option<i8>
    //pub deleted: bool,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[serde(default)]
pub struct Activity
{
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contentId: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub userId: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<i8>
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
    #[serde(default)]
    pub expensive: bool,
    pub limit: i64, //Everything is i64 so it's easier to serialize/deserialize
    pub skip: i64
}

#[macro_export]
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
        contentapi::Request {
            name: $name,
            r#type: $type.to_string(), //enum to string, because we implement display on all
            fields: $fields,
            query: $query,
            order: $order,
            limit: $limit.into(),
            skip: $skip.into(),
            expensive: false
        }
    };
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
