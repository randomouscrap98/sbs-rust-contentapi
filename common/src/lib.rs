pub mod forum;
pub mod layout;
pub mod pagination;
pub mod submission;
pub mod forum_render;

use std::collections::HashMap;

use bbscope::BBCode;
use chrono::{SecondsFormat, Utc};
use contentapi::{self, endpoints::{ApiError, ApiContext}, Content, Message, User, UserType};
use serde::{Serialize, Deserialize};
use serde_urlencoded;
use maud::{Markup, html, PreEscaped, DOCTYPE};

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
    pub theme: String
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            language: String::from("en"),
            compact: false,
            theme: String::from("sbs")
        }
    }
}

impl UserConfig {
    pub fn all_themes() -> Vec<(&'static str,&'static str)> {
        vec![
            ("sbs", "SBS (default)"),
            ("sbs-dark", "SBS Dark"),
            ("sbs-blue", "SBS Blue"),
            ("sbs-contrast", "SBS High Contrast"),
            ("sbs-dark-contrast", "SBS Dark High Contrast")
        ]
    }
}

#[derive(Debug)]
pub struct MainLayoutData {
    pub config: LinkConfig,     
    pub user_config: UserConfig,    
    pub current_path: String, 
    pub override_nav_path: Option<&'static str>,
    pub user: Option<contentapi::User>,
    pub user_token: Option<String>,
    pub about_api: contentapi::About, 

    #[cfg(feature = "profiling")]
    pub profiler: onestop::OneList<onestop::OneDuration>
}

/// A basic context for use in page rendering. Even if a page doesn't strictly need all
/// the items inside this context, it just makes it easier to pass them all to every page
/// render consistently. However, do NOT use this on the baseline rendering functions!
pub struct PageContext {
    pub layout_data: MainLayoutData,
    pub api_context: ApiContext,
    pub bbcode: BBCode,
    pub bbconsume: BBCode
}

// ------------------------
// *     GENERIC FORMS    *
// ------------------------

#[derive(Serialize, Deserialize, Debug)]
pub struct EmailGeneric
{
    pub email: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BasicText
{
    pub text: String
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

impl From<ApiError> for Error {
    fn from(error: ApiError) -> Self {
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
    //pub fn other(message: &str)
    pub fn to_user_string(&self) -> String {
        match self {
            Self::Api(error) => error.to_user_string(),
            Self::Other(error) => error.clone(),
            Self::NotFound(error) => error.clone(),
            Self::Data(error, _data) => error.clone()
        }
    }
}


// ------------------------
// *    LINK FUNCTIONS    *
// ------------------------

pub fn base_image_link(config: &LinkConfig, hash: &str) -> String { 
    image_link(config, hash, 0, false)
}

pub fn image_link(config: &LinkConfig, hash: &str, size: i64, crop: bool) -> String {
    let query = contentapi::QueryImage { 
        size : if size > 0 { Some(size as i64) } else { None },
        crop : if crop { Some(crop) } else { None }
    };
    match serde_urlencoded::to_string(&query) {
        Ok(querystring) => format!("{}/{}?{}", config.file_root, hash, querystring),
        Err(error) => {
            println!("Serde_qs failed? Not printing link for {}. Error: {}", hash, error);
            format!("#ERRORFOR-{}",hash)
        }
    }
}

/// This SHOULD work anywhere...
pub fn self_link(data: &MainLayoutData) -> String {
    format!("{}{}", data.config.http_root, data.current_path)
}

pub fn user_link(config: &LinkConfig, user: &User) -> String {
    format!("{}/user/{}", config.http_root, user.username)
}

pub fn page_link(config: &LinkConfig, page: &Content) -> String {
    format!("{}/page/{}", config.http_root, s(&page.hash))
}

pub fn forum_category_link(config: &LinkConfig, category: &Content) -> String {
    forum_category_link_unsafe(config, s(&category.hash))
}

/// Create a category link using the current link system, which only uses the hash AVOID AS MUCH AS POSSIBLE!
/// The implementation of the links may change!
pub fn forum_category_link_unsafe(config: &LinkConfig, hash: &str) -> String {
    format!("{}/forum/category/{}", config.http_root, hash) //s(&category.hash))
}

pub fn forum_thread_link(config: &LinkConfig, thread: &Content) -> String {
    format!("{}/forum/thread/{}", config.http_root, s(&thread.hash))
}

pub fn forum_post_hash(post: &Message) -> String {
    let post_id = post.id.unwrap_or(0);
    format!("#post_{}", post_id)
}

pub fn forum_post_link(config: &LinkConfig, post: &Message, thread: &Content) -> String {
    let post_id = post.id.unwrap_or(0);
    format!("{}/forum/thread/{}/{}{}", config.http_root, s(&thread.hash), post_id, forum_post_hash(post))
}

// ----------------------------
// *     FORMAT FUNCTIONS     *
// ----------------------------

pub fn timeago(time: &chrono::DateTime<chrono::Utc>) -> String {
    let duration = chrono::Utc::now().signed_duration_since(*time); //timeago::format()
    match duration.to_std() {
        Ok(stdur) => {
            timeago::format(stdur, timeago::Style::HUMAN)
        },
        Err(error) => {
            format!("PARSE-ERR({}):{}", duration, error)
        }
    }
}

pub fn timeago_o(time: &Option<chrono::DateTime<chrono::Utc>>) -> String {
    if let Some(time) = time {
        timeago(time)
    }
    else {
        String::from("???")
    }
}

pub fn is_empty(string: &Option<String>) -> bool {
    if let Some(s) = string { s.is_empty() }
    else { true }
}

pub fn s(string: &Option<String>) -> &str {
    if let Some(s) = string { &s }
    else { "" }
}

pub fn b(boolean: bool) -> &'static str {
    if boolean { "true" }
    else { "false" }
}

pub fn d(date: &Option<chrono::DateTime<Utc>>) -> String {
    if let Some(date) = date { dd(date) }
    else { String::from("NODATE") }
}

pub fn dd(date: &chrono::DateTime<Utc>) -> String {
    date.to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub fn i(int: &Option<i64>) -> String {
    if let Some(int) = int { format!("{}", int) }
    else { String::from("??") }
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
            createDate: chrono::Utc::now()
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

// ---------------------
// *    FRAGMENTS      *
// ---------------------

pub fn style(config: &LinkConfig, link: &str) -> Markup {
    html! {
        link rel="stylesheet" href={(config.static_root) (link) "?" (config.cache_bust) } { }
    }
}

pub fn script(config: &LinkConfig, link: &str) -> Markup {
    html! {
        script src={(config.static_root) (link) "?" (config.cache_bust) } defer { }
    }
}

pub fn errorlist(errors: Option<Vec<String>>) -> Markup {
    html! {
        div."errorlist" {
            @if let Some(errors) = errors {
                @for error in errors {
                    div."error" {(error)}
                }
            }
        }
    }
}


// Produce some metadata for the header that any page can use (even widgets)
pub fn basic_meta(config: &LinkConfig) -> Markup{
    html! {
        //Can I have comments in html markup?
        meta charset="UTF-8";
        meta name="rating" content="general";
        meta name="viewport" content="width=device-width";
        //[] is for optional, {} is for concatenate values
        link rel="icon" type="image/svg+xml" sizes="any" href={(config.resource_root) "/favicon.svg"};
    } 
}