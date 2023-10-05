pub mod forum;
pub mod render;
pub mod pagination;
pub mod search;
pub mod constants;
pub mod forms;
pub mod links;
pub mod view;
pub mod prefab;
pub mod response;

use std::collections::HashMap;

use maud::*;
use serde::{Serialize, Deserialize};
use serde_urlencoded;

use bbscope::BBCode;
use contentapi::*;
use fastrand;

#[macro_export]
macro_rules! opt_s {
    ($str:expr,$def:literal) => {
        if let Some(ref thing) = $str { if thing.trim().is_empty() { $def } else { thing } } else { $def }
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
    pub toppagination_posts: bool,
    pub theme: String,
    //pub shadows: bool
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            language: String::from("en"),
            compact: false,
            toppagination_posts: false,
            theme: String::from("sbs"),
            //shadows: false
        }
    }
}

#[derive(Debug)]
pub struct MainLayoutData {
    pub links: LinkConfig,     
    pub user_config: UserConfig,    
    /// Should be the path ONLY, no machine or query. If it's not that, it's an error!
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

pub fn random_id(postfix: &str) -> String {
    format!("{}_{}", fastrand::u32(..), postfix)
}