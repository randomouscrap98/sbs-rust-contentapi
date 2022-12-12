
pub mod index;
pub mod about;
pub mod login;
pub mod activity;
pub mod search;
pub mod widget_imagebrowser;
pub mod widget_bbcodepreview;
pub mod userhome;
pub mod recover;
pub mod register;
pub mod registerconfirm;
pub mod user;
pub mod _forumsys; //non-page mod
pub mod forum_main;
pub mod forum_category;
pub mod forum_thread;

use chrono::{SecondsFormat, Utc};
use contentapi::{self, endpoints::ApiError, Content, Message, User, UserType};
use serde::{Serialize, Deserialize};
use serde_urlencoded;
use maud::{Markup, html, PreEscaped, DOCTYPE};

#[derive(Clone, Debug)]
pub struct LinkConfig {
    pub http_root: String,
    pub static_root: String,
    pub resource_root: String,
    pub file_root: String,
    pub cache_bust: String
}

#[derive(Clone, Debug)]
pub struct UserConfig {
    pub language: String
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            language: String::from("en")
        }
    }
}

#[derive(Debug)]
pub struct MainLayoutData {
    pub config: LinkConfig,     
    pub user_config: UserConfig,    
    pub current_path: String, 
    pub user: Option<contentapi::User>,
    pub about_api: contentapi::About, 
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

pub fn user_link(config: &LinkConfig, user: &User) -> String {
    format!("{}/user/{}", config.http_root, user.username)
}

pub fn forum_category_link(config: &LinkConfig, category: &Content) -> String {
    let hash = match &category.hash {
        Some(hash) => hash.clone(),
        None => String::from("")
    };
    format!("{}/forum/category/{}", config.http_root, hash)
}

pub fn forum_thread_link(config: &LinkConfig, thread: &Content) -> String {
    let hash = match &thread.hash {
        Some(hash) => hash.clone(),
        None => String::from("")
    };
    format!("{}/forum/thread/{}", config.http_root, hash) //}"{{@root.http_root}}/forum/thread/{{thread.hash}}" class="flatlink">{{thread.name}}</a> }
}

pub fn forum_post_link(config: &LinkConfig, post: &Message, thread: &Content) -> String {
    let post_id = post.id.unwrap_or(0);
    let hash = match &thread.hash {
        Some(hash) => hash.clone(),
        None => String::from("")
    };
    format!("{}/forum/thread/{}/{}#post_{}", config.http_root, hash, post_id, post_id)
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
    if let Some(date) = date { date.to_rfc3339_opts(SecondsFormat::Secs, true) }
    else { String::from("NODATE") }
}

pub fn i(int: &Option<i64>) -> String {
    if let Some(int) = int { format!("{}", int) }
    else { String::from("??") }
}

//Email errors are weird with their true/false return. 
macro_rules! email_errors {
    ($result:expr) => {
        {
            let mut errors: Vec<String> = Vec::new();
            match $result //post_sendemail(context, email).await
            {
                //If confirmation is successful, we get a token back. We login and redirect to the userhome page
                Ok(success) => {
                    if !success {
                        errors.push(String::from("Unkown error (email endpoint returned false!)"));
                    }
                },
                //If there's an error, we re-render the confirmation page with the errors.
                Err(error) => {
                    println!("Email endpoint raw error: {}", error.to_verbose_string());
                    errors.push(error.to_user_string());
                } 
            }
            errors
        }
    };
}
pub(crate) use email_errors;

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


// ---------------------
// *    FRAGMENTS      *
// ---------------------

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

//Render basic navigation link with only text as the body
pub fn main_nav_link(config: &LinkConfig, text: &str, href: &str, current_path: &str, id: Option<&str>) -> Markup {
    main_nav_link_raw(config, PreEscaped(String::from(text)), href, current_path, id)
}

//Produce a link for site navigation which supports highlighting if on current page. Body can be "anything"
pub fn main_nav_link_raw(config: &LinkConfig, body: Markup, href: &str, current_path: &str, id: Option<&str>) -> Markup {
    let mut class = String::from("plainlink headertab");
    if current_path.starts_with(href) { class.push_str(" current"); }
    html! {
        a.(class) href={(config.http_root) (href)} id=[id] { (body) }
    }
}

//Produce just the inner user element (not the link itself) for a logged-in user
pub fn header_user_inner(config: &LinkConfig, user: &contentapi::User) -> Markup {
    html! {
        span { (user.username) }
        img src=(image_link(config, &user.avatar, 100, true));
    }
}

pub fn header(config: &LinkConfig, current_path: &str, user: &Option<contentapi::User>) -> Markup {
    html! {
        header."controlbar" {
            nav {
                a."plainlink" #"homelink" href={(config.http_root)"/"}{
                    img src={(config.resource_root)"/favicon.ico"};
                    (main_nav_link(config,"Activity","/activity",current_path,None))
                    (main_nav_link(config,"Browse","/search",current_path,None))
                    (main_nav_link(config,"Forums","/forum",current_path,None))
                }
            }
            div #"header-user" {
                @if let Some(user) = user {
                    (main_nav_link_raw(config,header_user_inner(config,user),"/userhome",current_path,None))
                }
                @else {
                    (main_nav_link(config,"Login","/login",current_path,None))
                }
            }
        }
    }
}

//Produce the footer for the main selection of pages
pub fn footer(config: &LinkConfig, about_api: &contentapi::About, current_path: &str) -> Markup {
    html! {
        footer class="controlbar smallseparate" {
            span #"api_about" { (about_api.environment) " - " (about_api.version) }
            (main_nav_link(config,"About","/about",current_path,Some("footer-about")))
            //<!--<span id="debug">{{client_ip}}</span>-->
            //<!--<span id="debug">{{route_path}}</span>-->
        }
    }
}


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

pub fn layout(main_data: &MainLayoutData, page: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang=(main_data.user_config.language) {
            head {
                (basic_meta(&main_data.config))
                title { "SmileBASIC Source" }
                meta name="description" content="A community for sharing programs and getting advice on SmileBASIC applications on the Nintendo DSi, 3DS, and Switch";
                (style(&main_data.config, "/base.css"))
                (style(&main_data.config, "/layout.css"))
                (script(&main_data.config, "/sb-highlight.js"))
                (script(&main_data.config, "/base.js"))
                (script(&main_data.config, "/layout.js"))
                style { (PreEscaped(r#"
                    body {
                        background-repeat: repeat;
                        background-image: url(""#))(main_data.config.resource_root)(PreEscaped(r#"/sb-tile.png")
                    }
                    "#))
                }
            }
        }
        body {
            (header(&main_data.config, &main_data.current_path, &main_data.user))
            main { (page) }
            (footer(&main_data.config, &main_data.about_api, &main_data.current_path ))
        }
    }
}