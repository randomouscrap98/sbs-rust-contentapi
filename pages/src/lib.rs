
pub mod index;
pub mod about;
pub mod login;
pub mod activity;
pub mod search;
pub mod widget_imagebrowser;
pub mod userhome;

use contentapi::{self, endpoints::ApiError};
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
    Other(String) //Something "general" happened, who the heck knows?
}

impl From<ApiError> for Error {
    fn from(error: ApiError) -> Self {
        Error::Api(error) 
    }
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        Error::Other(error.to_string()) 
    }
}

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

///// A macro to convert an API error into a simple rendered response. The "place"
///// you pass in is used both for printing an error AND for finding the appropriate render function
//macro_rules! emit_error_response {
//    ($place:path,$error:expr,$) => {
//        {
//            println!("{} raw error: {}", stringify!(place), error.to_verbose_string());
//            Response::Render($place::render(data, Some(vec![error.to_user_string()]), None, None))
//        }
//    };
//}


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

pub fn threadicon(config: &LinkConfig, neutral: bool, sticky: bool, locked: bool) -> Markup {
    html! {
        div."threadicon" {
            @if neutral { img src={(config.resource_root)"/sb-page.png"}; }
            @if sticky { span{"📌"} }
            @if locked { span{"🔒"} }
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
                (script(&main_data.config, "/layout.js"))
                (script(&main_data.config, "/base.js"))
                (script(&main_data.config, "/sb-highlight.js"))
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