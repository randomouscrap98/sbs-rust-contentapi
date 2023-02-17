pub mod forum;
pub mod render;
pub mod pagination;
pub mod submissions;
pub mod admin;
pub mod constants;
pub mod forms;
pub mod links;

use std::collections::HashMap;

use maud::*;
use serde::{Serialize, Deserialize};
use serde_urlencoded;

use bbscope::BBCode;
use contentapi::*;

#[macro_export]
macro_rules! opt_s {
    ($str:expr,$def:literal) => {
        if let Some(ref thing) = $str { thing } else { $def }
    };
    ($str:expr) => {
        if let Some(ref thing) = $str { thing } else { "" }
    };
}

#[derive(Clone, Debug)]
pub struct LinkConfig {
    pub http_root: String,
    pub static_root: String,
    pub resource_root: String,
    pub file_root: String,
    pub file_upload_root: String,
    pub cache_bust: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct UserConfig {
    pub language: String,
    pub compact: bool,
    pub theme: String,
    //pub shadows: bool
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            language: String::from("en"),
            compact: false,
            theme: String::from("sbs"),
            //shadows: false
        }
    }
}

#[derive(Debug)]
pub struct MainLayoutData {
    pub links: LinkConfig,     
    pub user_config: UserConfig,    
    pub current_path: String, 
    pub override_nav_path: Option<&'static str>,
    pub user: Option<contentapi::User>,
    pub user_token: Option<String>,
    pub about_api: contentapi::About, 
    pub raw_alert: Option<String>,

    #[cfg(feature = "profiling")]
    pub profiler: onestop::OneList<onestop::OneDuration>
}

/// A basic context for use in page rendering. Even if a page doesn't strictly need all
/// the items inside this context, it just makes it easier to pass them all to every page
/// render consistently. However, do NOT use this on the baseline rendering functions!
pub struct PageContext {
    pub layout_data: MainLayoutData,
    pub api_context: endpoints::ApiContext,
    pub bbcode: BBCode
}

// -------------------------------------
// *     Response/Error from pages     *
// -------------------------------------

#[derive(Debug)]
pub enum Response {
    Render(String), //string is the markup
    Redirect(String)
}

#[derive(Debug)]
pub enum Error {
    Api(contentapi::endpoints::ApiError),
    Data(String, String), //First string is error to output, second is the data itself (don't print for user)
    NotFound(String),   //Normal "not found" error
    Other(String) //Something "general" happened, who the heck knows?
}

impl From<endpoints::ApiError> for Error {
    fn from(error: endpoints::ApiError) -> Self {
        Error::Api(error) 
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::Other(error.to_string()) 
    }
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        Error::Other(error.to_string()) 
    }
}

impl Error {
    pub fn to_user_string(&self) -> String {
        match self {
            Self::Api(error) => error.to_user_string(),
            Self::Other(error) => error.clone(),
            Self::NotFound(error) => error.clone(),
            Self::Data(error, _data) => error.clone()
        }
    }
}


// --------------------------
// *    Helper utilities    *
// --------------------------

pub fn is_empty(string: &Option<String>) -> bool {
    if let Some(s) = string { s.is_empty() }
    else { true }
}

pub fn user_or_default(user: Option<&User>) -> User {
    if let Some(u) = user {
        u.clone()
    }
    else {
        User {
            username: String::from("???"),
            id: 0,
            avatar: String::from("0"),
            r#type: UserType::USER,
            admin: false,
            special: None,
            createDate: chrono::Utc::now(),
            groups: Vec::new()
        }
    }
}

pub fn get_user_or_default(uid: Option<i64>, users: &HashMap<i64, User>) -> User {
    user_or_default(users.get(&uid.unwrap_or(0)))
    //user_or_default(users.get(&uid.unwrap_or(0)))
}

pub fn content_or_default(content: Option<&Content>) -> Content {
    if let Some(c) = content {
        c.clone()
    }
    else {
        let mut result = Content::default();
        result.hash = Some(String::from("#"));
        result.name = Some(String::from("???"));
        result.createUserId = Some(0);
        result
    }
}

/// Parse a comma or space separated string into parts
pub fn parse_compound_value(original: &str) -> Vec<String>
{
    //First, convert all commas into spaces
    let cleansed = original.replace(",", " ");

    //Then, split by space
    let mut result = Vec::new();
    for v in cleansed.split_ascii_whitespace() {
        result.push(v.to_string())
    }

    result
}
