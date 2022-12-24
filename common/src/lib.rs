pub mod forum;
pub mod render;
pub mod pagination;
pub mod submissions;
pub mod queries;
pub mod constants;
pub mod data;
pub mod forms;
pub mod links;

use std::collections::HashMap;

use bbscope::BBCode;
use contentapi::{self, endpoints::{ApiError}, Content,  User, UserType};
use serde::{Serialize, Deserialize};
use serde_urlencoded;
use maud::{Markup, html, PreEscaped, DOCTYPE};

use data::*;

#[macro_export]
macro_rules! opt_s {
    ($str:expr,$def:literal) => {
        if let Some(ref thing) = $str { thing } else { $def }
    };
    ($str:expr) => {
        if let Some(ref thing) = $str { thing } else { "" }
    };
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
