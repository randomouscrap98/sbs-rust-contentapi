use chrono::{DateTime, Utc};
use core::fmt::Debug;
use hyper;
use maud::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::common::contentapi::*;
use super::common::render::*;
use super::common::*;
use super::context::*;
use crate::layout::*;
use crate::links::*;
use crate::response::*;
use crate::*;

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ThreadQuery {
    pub reply: Option<i64>,
    pub selected: Option<i64>,
}

// -----------------------------------------
//             TEMP
// -----------------------------------------

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

/// This is needed so often: just convert any generic error into a "no request" error,
/// assuming you have the AboutRequest...
macro_rules! noreqerr {
    ($result:expr, $req:ident) => {
        $result.map_err(|e| ApiError::NonRequest($req.clone(), e.to_string()))
    };
}

/// This isn't needed as often: just convert any generic error into a "network" error
macro_rules! neterr {
    ($result:expr, $req:ident) => {
        $result.map_err(|e| ApiError::Network($req.clone(), e.to_string()))
    };
}

/// This isn't needed as often: just convert any generic error into a "parse" error
macro_rules! parseerr {
    ($result:expr, $req:ident) => {
        parseerr!($result, $req, None)
    };
    ($result:expr, $req:ident, $data:expr) => {
        $result.map_err(|e| ApiError::Parse($req.clone(), e.to_string(), $data))
    };
}

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
    };
}

/// Cast result; if key doesn't exist, you get none.
pub fn cast_result<T>(
    result: &RequestResult,
    name: &str,
) -> Result<Option<Vec<T>>, Box<dyn std::error::Error>>
where
    T: for<'a> Deserialize<'a>,
{
    //If the key exists, do the conversion
    if let Some(content) = result.objects.get(name) {
        let mut items: Vec<T> = Vec::new();
        for c in content {
            items.push(<T as Deserialize>::deserialize(c)?);
        }
        Ok(Some(items))
    } else {
        Ok(None)
    }
}

/// Cast result without care if the key exists. You'll get an empty vector
pub fn cast_result_safe<T>(
    result: &RequestResult,
    name: &str,
) -> Result<Vec<T>, Box<dyn std::error::Error>>
where
    T: for<'a> Deserialize<'a>,
{
    cast_result(result, name).and_then(|r| Ok(r.unwrap_or(Vec::new())))
}

/// Cast result but throw error if the key isn't found
pub fn cast_result_required<T>(
    result: &RequestResult,
    name: &str,
) -> Result<Vec<T>, Box<dyn std::error::Error>>
where
    T: for<'a> Deserialize<'a>,
{
    cast_result(result, name)?.ok_or(format!("Couldn't find key {}", name).into())
}

#[derive(Debug)]
pub enum ApiError {
    NonRequest(AboutRequest, String), //Something not pertaining to the actual request itself happened!
    Parse(AboutRequest, String, Option<Vec<u8>>), //Something didn't parse correctly! This is common enough to be its own error
    Network(AboutRequest, String), //Is the API reachable? Endpoint not necessary most likely; this indicates an error beyond 404
    Request(AboutRequest, String, u16), //Oh something went wrong with the request itself! Probably a 400 or 500 error
    Other(String),                      //Avoid this at all costs, if you can
}

impl ApiError {
    pub fn to_user_string(&self) -> String {
        match self {
            Self::NonRequest(_, err) => err.clone(),
            Self::Parse(_, err, _) => err.clone(),
            Self::Network(_, err) => err.clone(),
            Self::Request(_, err, _) => err.clone(), //May change?
            Self::Other(err) => err.clone(),
        }
    }
    pub fn to_status(&self) -> u16 {
        match self {
            Self::NonRequest(_, _) => 500,
            Self::Parse(_, _, _) => 500,
            Self::Network(_, _) => 503,
            Self::Request(_, _, _) => 400,
            Self::Other(_) => 500,
        }
    }
    pub fn to_verbose_string(&self) -> String {
        match self {
            Self::NonRequest(about, err) => format!(
                "[{}]{} - Something happened before we could reach the backend: {}",
                about.verb, about.endpoint, err
            ),
            Self::Parse(about, err, data) => {
                if let Some(data) = data {
                    format!(
                        "[{}]{} - Couldn't parse response from backend: {}. Data:\n{}",
                        about.verb,
                        about.endpoint,
                        err,
                        String::from_utf8_lossy(data).into_owned()
                    )
                } else {
                    format!("[{}]{} - 'ParseError': COULDN'T GET BYTES FROM BODY OF RESPONSE (THIS IS BAD!): {}", about.verb, about.endpoint, err)
                }
            }
            Self::Network(about, err) => format!(
                "[{}]{} - The backend seems to be unreachable: {}",
                about.verb, about.endpoint, err
            ),
            Self::Request(about, err, api_status_code) => format!(
                "[{}]{} - Bad request to API ({}): {}",
                about.verb, about.endpoint, api_status_code, err
            ),
            Self::Other(err) => format!("Generic API error: {}", err),
        }
    }
}

impl From<ApiError> for Error {
    fn from(err: ApiError) -> Self {
        // Here we map the inner details of MyError to AnotherError
        match err {
            ApiError::NonRequest(_, s) => Error::Other(s),
            ApiError::Other(s) => Error::Other(s),
            ApiError::Network(_, s) => Error::Other(s),
            ApiError::Parse(_, s, _) => Error::Other(s),
            ApiError::Request(_, s, _) => Error::Other(s),
        }
    }
}

impl From<Box<dyn std::error::Error>> for ApiError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        Self::Other(error.to_string())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub id: i64,
    pub r#type: i8,
    pub username: String,
    pub avatar: String,
    pub special: Option<String>,
    //Note that we used "alias" before, but that's just for deserialization. Rename is what you want for both
    // #[serde(rename = "super", deserialize_with = "deserialize_bool_from_anything")]
    // pub admin: bool,
    pub createDate: DateTime<Utc>,
    pub groups: Vec<i64>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[serde(default)]
pub struct Message {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contentId: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub createUserId: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub createDate: Option<DateTime<Utc>>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_literalType: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_contentType: Option<i8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestResult {
    pub search: FullRequest,
    pub databaseTimes: HashMap<String, f64>,
    pub objects: HashMap<String, Vec<serde_json::Value>>,
    pub totalTime: f64,
    pub nonDbTime: f64,
    pub requestUser: Option<i64>,
}

//Note: you WANT all these strings to be owned, even if it wastes memory or whatever,
//because you want to be able to construct and pass around whatever requests you want
//from anywhere to anywhere and have the lifetimes of the internals strongly tied to the
//struct. It's about HOW you're using the struct, not simply "saving memory".
//#[serde_with::skip_serializing_none] //MUST COME BEFORE
#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
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
    pub skip: i64,
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
        build_request!(
            $type,
            $fields,
            Some($query),
            Some($order),
            $limit,
            $skip,
            None
        )
    };
    ($type:expr, $fields:expr, $query:expr, $order:expr, $limit:expr, $skip:expr, $name:expr) => {
        Request {
            name: $name,
            r#type: $type.to_string(), //enum to string, because we implement display on all
            fields: $fields,
            query: $query,
            order: $order,
            limit: $limit.into(),
            skip: $skip.into(),
            expensive: false,
        }
    };
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FullRequest {
    pub values: HashMap<String, serde_json::Value>, //HashMap<String, Box<Serialize>>,
    pub requests: Vec<Request>,
}

impl FullRequest {
    pub fn new() -> Self {
        FullRequest {
            values: HashMap::new(),
            requests: Vec::new(),
        }
    }
}

//You'll want to create a new api context to make multiple requests, as it's more efficient.
//Maybe one per request?
pub struct ApiContext {
    api_url: String,
    client: hyper::client::Client<hyper::client::HttpConnector>,
}

impl ApiContext {
    pub fn new(api_url: String) -> Self {
        Self {
            api_url,
            client: hyper::client::Client::new(),
        }
    }

    pub fn get_endpoint(&self, endpoint: &str) -> String {
        format!("{}{}", self.api_url, endpoint)
    }

    /// All requests to the API start off the same
    fn get_request_builder(
        &self,
        request: &AboutRequest,
        method: hyper::Method,
    ) -> Result<hyper::http::request::Builder, ApiError> {
        let endpoint_uri = noreqerr!(
            self.get_endpoint(&request.endpoint).parse::<hyper::Uri>(),
            request
        )?;

        let reqbuilder = hyper::Request::builder()
            .method(method)
            .uri(endpoint_uri)
            .header("Accept", "application/json");

        Ok(reqbuilder)
    }

    //Once a response comes back from the API, figure out the appropriate errors or data to parse and return
    async fn handle_response<T: DeserializeOwned>(
        response: hyper::Response<hyper::Body>,
        about: AboutRequest,
    ) -> Result<T, ApiError> {
        let status = response.status();
        let u_status = status.as_u16();

        let body = parseerr!(hyper::body::to_bytes(response.into_body()).await, about)?;

        //Good status vs all the rest.
        if status.is_success() {
            //At this point, the body isn't needed anymore, since the json will have run before we
            //call Some(body.into())
            parseerr!(serde_json::from_slice::<T>(&body), about, Some(body.into()))
        } else {
            match String::from_utf8(body.into_iter().collect()) {
                Ok(error) => Err(ApiError::Request(about, error, u_status)),
                Err(error) => Err(ApiError::Request(
                    about,
                    format!("RESPONSE BODY UTF-8 ERROR: {}", error),
                    u_status,
                )),
            }
        }
    }

    //Construct a basic POST request to the given endpoint (including ?params) using the given
    //request context. Automatically add bearer headers and all that
    pub async fn basic_post_request<U: Serialize + Debug, T: DeserializeOwned>(
        &self,
        request: AboutRequest,
        data: &U,
    ) -> Result<T, ApiError> {
        let reqbuilder = self
            .get_request_builder(&request, hyper::Method::POST)?
            .header("Content-Type", "application/json");
        let json = noreqerr!(serde_json::ser::to_string(data), request)?; //Even though this is serde, it's not a parse error because it's before the request
        let req = noreqerr!(reqbuilder.body(hyper::Body::from(json)), request)?;

        let response = self
            .client
            .request(req)
            .await
            .map_err(|e| ApiError::Network(request.clone(), e.to_string()))?;

        Self::handle_response(response, request).await
    }
}

macro_rules! make_post_endpoint {
    ($name:ident<$intype:ty,$type:ty>($endpoint:literal)) => {
        pub async fn $name(&self, data: &$intype) -> Result<$type, ApiError> {
            self.basic_post_request(
                AboutRequest {
                    endpoint: String::from($endpoint),
                    verb: String::from("POST"),
                    post_data: Some(format!("{:#?}", data)),
                },
                data,
            )
            .await
        }
    };
}

//This is the rest of the implementation, which are all the actual functions you want to call!
impl ApiContext {
    make_post_endpoint! {post_request<FullRequest,RequestResult>("/request")}
}

string_enum! { RequestType => {
    user,
    content,
    message,
    activity,
    watch,
    message_aggregate,
    activity_aggregate,
    content_engagement,
    ban,
    keyword_aggregate,
    message_engagement,
    userrelation
}}

#[derive(Debug, Clone)]
pub struct AboutRequest {
    //This is GET/POST/etc. I don't care for it to be an enum, since I'm just printing it
    pub verb: String,
    pub endpoint: String,
    //Restricted data, which should probably not even be logged to the console! So what do
    //we do with it? It's mostly just for debugging I think, there may be a flag to enable
    //printing the restricted data
    pub post_data: Option<String>,
}

/// Generate a request for ONLY messages and users for the given root post id. NO limits set on reply chain
/// length (other than those imposed by the API)
// pub fn get_reply_request(root_post_id: i64) -> FullRequest {
//     //NOTE: valuein WAY WAY faster than valuelike! always prefer it!
//     let mut request = get_generic_message_request(
//         "!valuein(@root_key,@root_post) or id = @root_post",
//         Vec::new(),
//         0,
//         0,
//     );
//     add_value!(request, "root_key", vec!["re-top"]);
//     add_value!(request, "root_post", vec![root_post_id]);
//     request
// }

/// Render a single post on a thread.
pub struct PostsConfig {
    /// The thread that holds all the posts to render
    pub thread: ForumThread2,
    pub categories: Vec<SubmissionCategory>,
    //pub posts: Vec<Message>,
    pub posts: Vec<ForumPost>,
    pub related: HashMap<i64, ForumPost>,
    //pub related: HashMap<i64, Message>,
    //pub users: HashMap<i64, User>,
    pub users: HashMap<i64, User2>,
    /// The path to this thread; if not given, path not rendered. Thread must also be given
    pub path: Option<Vec<ForumPathItem>>,
    /// The pages to navigate posts; not displayed if not given
    pub pages: Option<Vec<PagelistItem>>,
    pub start_num: Option<i32>,
    pub selected_post_id: Option<i64>,
    //pub docs_content: Option<Vec<Content>>, //DocTreeNode<'a>>,
    pub render_header: bool,
    pub render_page: bool,
    pub render_reply_chain: bool,
    pub render_reply_link: bool,
    pub render_controls: bool, //pub render_sequence: bool
}

impl PostsConfig {
    pub fn thread_mode(
        thread: ForumThread2,
        categories: Vec<SubmissionCategory>,
        posts: Vec<ForumPost>,
        related: HashMap<i64, ForumPost>,
        users: HashMap<i64, User2>,
        path: Vec<ForumPathItem>,
        pages: Vec<PagelistItem>,
        start: i32,
        selected_post_id: Option<i64>,
    ) -> Self {
        //let posts = thread.posts.clone();
        Self {
            thread,
            related,
            users,
            posts,
            categories,
            path: Some(path),
            pages: Some(pages),
            start_num: Some(start),
            selected_post_id,
            render_header: true,
            render_page: true,
            render_reply_chain: false,
            render_reply_link: true,
            render_controls: true,
            //docs_content: None,
        }
    }
    pub fn reply_mode(
        thread: ForumThread2,
        posts: Vec<ForumPost>,
        related: HashMap<i64, ForumPost>,
        users: HashMap<i64, User2>,
        selected_post_id: Option<i64>,
    ) -> Self {
        //let posts = thread.posts.clone();
        Self {
            thread,
            related,
            users,
            posts,
            categories: Vec::new(),
            path: None,
            pages: None,
            start_num: None,
            selected_post_id,
            render_header: false,
            render_page: false,
            render_reply_chain: true,
            render_reply_link: false,
            render_controls: false,
            //docs_content: None,
        }
    }
}

pub const SHORTDESCRIPTION: usize = 200;

pub fn short_post(message: &ForumPost) -> String {
    message
        .text
        .chars()
        .take(SHORTDESCRIPTION)
        .collect::<String>()
}

pub fn short_description2(thread: &ForumThread2) -> String {
    if !thread.description.is_empty() {
        return thread.description.clone();
    }
    if thread.literal_type != SBSPageType::FORUMTHREAD {
        //Get some short portion of the body, even if it's bad? We'll fix bbcode stuff later
        if !thread.text.is_empty() {
            return thread
                .text
                .chars()
                .take(SHORTDESCRIPTION)
                .collect::<String>();
        }
    }
    return String::from("");
}

fn images_to_attr(config: &LinkConfig, images: &Vec<String>) -> String {
    serde_json::to_string(
        &images
            .iter()
            .map(|i| config.image_default(i))
            .collect::<Vec<String>>(),
    )
    .unwrap_or_else(|err| {
        println!("ERROR: COULD NOT SERIALIZE PAGE IMAGES: {}", err);
        String::new()
    })
}

/// Render the page data, such as text and infoboxes, on standard pages. True forum threads don't have main
/// content like that, so this is only called on programs, resources, etc
pub fn render_page(
    data: &MainLayoutData,
    bbcode: &mut BBCode,
    thread: &ForumThread2,
    categories: &Vec<SubmissionCategory>,
) -> Markup {
    let values = &thread.values;
    let systems = get_systems2(&values);

    let images: Option<Vec<String>> = get_value_safe!(values, SBSValue::IMAGES, Vec<String>);
    let dlkey: Option<String> = get_value_safe!(values, SBSValue::DOWNLOADKEY, String);
    let version: Option<String> = get_value_safe!(values, SBSValue::VERSION, String);
    let size: Option<String> = get_value_safe!(values, SBSValue::SIZE, String);

    html! {
        section {
            //First check is if it's a program, then we float this box to the right
            @if thread.literal_type == SBSPageType::PROGRAM {
                div."programinfo" {
                    @if let Some(images) = images {
                        div."gallery" #"page_gallery" /*data-index="0"*/ data-images=(images_to_attr(&data.links, &images)) {
                            //we now have the images: we just need the first one (it's a hash?)
                            @if let Some(image) = images.get(0) {
                                img src=(data.links.image_default(image));
                            }
                        }
                    }
                    div."extras mediumseparate" {
                        @if let Some(key) = dlkey {
                            span."smallseparate" {
                                b { "Download:" }
                                span."key" { (key) }
                                (threadicon2(&data.links, &thread))
                            }
                        }
                        @if systems.iter().any(|s| s == &PTCSYSTEM) {
                            span."smallseparate" {
                                b { "Download:" }
                                a."key" href=(data.links.qr_generator_unsafe(&thread.hash)) { "QR Codes" }
                                (threadicon2(&data.links, &thread))
                            }
                        }
                        @if let Some(version) = version {
                            span."smallseparate" {
                                b { "Version:" }
                                span."version" { (version) }
                            }
                        }
                        @if let Some(size) = size {
                            span."smallseparate" {
                                b { "Size:" }
                                span."size" { (size) }
                            }
                        }
                    }
                }
            }
            (render_textcontent(&thread.text, &thread.values, bbcode))
            //Documentation has no categories
            @if thread.literal_type != SBSPageType::DOCUMENTATION {
                hr."smaller";
                div."categorylist smallseparate" {
                    @for category in categories {
                        a."flatlink" href=(data.links.search_category(category.id)) { (category.name) }
                    }
                }
            }
        }
    }
}

pub fn render_textcontent(
    text: &str,
    values: &HashMap<String, String>,
    bbcode: &mut BBCode,
) -> Markup {
    let mut markup: &str = MARKUPBBCODE;
    if let Some(mkup) = get_value_safe!(values, SBSValue::MARKUP, &str) {
        markup = mkup;
    }
    html!(
        div."content" data-markup=(markup) data-prerendered[markup == MARKUPBBCODE] {
            @if markup == MARKUPBBCODE {
                (PreEscaped(&bbcode.parse(text)))
            }
            @else {
                (text)
            }
        }
    )
}

// #[derive(Serialize, Deserialize, Default, Clone, Debug)]
// #[serde(default)]
// pub struct Message {
//     pub id: Option<i64>,
//     pub contentId: Option<i64>,
//     pub createUserId: Option<i64>,
//     pub createDate: Option<DateTime<Utc>>,
//     pub text: Option<String>,
//     pub values: Option<HashMap<String, serde_json::Value>>,
//     pub engagement: Option<HashMap<String, HashMap<String, i64>>>,
//     pub editDate: Option<DateTime<Utc>>,
//     pub editUserId: Option<i64>,
//     pub module: Option<String>,
//     pub content_literalType: Option<String>,
//     pub content_contentType: Option<i8>,
// }

pub static MSGFIELDS: &str = "";
pub static MSGVALUESELECT: &str = "SELECT `key`,`value` FROM message_values WHERE messageId=?";

#[derive(Clone, Debug)]
pub struct ForumPost {
    pub id: i64,
    pub content_id: i64,
    pub create_user_id: i64,
    pub create_date: DateTime<Utc>,
    pub text: String,
    pub edit_date: Option<DateTime<Utc>>,
    pub edit_user_id: Option<i64>,
    pub values: HashMap<String, String>,
}

pub fn gather_forum_posts(
    query: &str,
    ctx: &PageContext,
    params: &[&dyn rusqlite::ToSql],
) -> Result<Vec<ForumPost>, Error> {
    let mut stmt = ctx.dbcon.prepare(query)?;
    let mut vstmt = ctx.dbcon.prepare(MSGVALUESELECT)?;
    easy_query((&mut stmt, query), params, |row| {
        let id = row.get(0)?;
        Ok(ForumPost {
            id,
            content_id: row.get(1)?,
            create_user_id: row.get(2)?,
            create_date: row.get(3)?,
            text: row.get(4)?,
            edit_date: row.get(5)?,
            edit_user_id: row.get(6)?,
            values: gather_values(&mut vstmt, id)?,
        })
    })
}

static COMMONMESSAGE: &str = "m.deleted = 0 AND m.module IS NULL AND m.contentId IN (SELECT contentId FROM content_permissions WHERE read=1 AND userId=0)";

fn forum_posts_base_query() -> (String, Vec<Box<dyn rusqlite::ToSql>>) {
    let query = format!(
        "SELECT {} FROM messages m WHERE {}",
        "m.id,m.contentId,m.createUserId,m.createDate,m.text,m.editDate,m.editUserId",
        COMMONMESSAGE,
    );
    let params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![]; //Box::new(ContentType::PAGE)];
    (query, params)
}

fn get_forumposts_basic(
    ctx: &PageContext,
    content_id: i64,
    ids: Option<Vec<i64>>,
    limit: QueryLimit,
) -> Result<Vec<ForumPost>, Error> {
    let (mut query, mut params) = forum_posts_base_query();
    query.push_str(" AND m.contentId = ?");
    params.push(Box::new(content_id));
    if let Some(ids) = ids {
        query.push_str(" AND m.id IN (");
        query.push_str(&params_list(ids.len()));
        query.push_str(")");
        for id in ids {
            params.push(Box::new(id));
        }
    }
    limit.mod_query(&mut query, &mut params);
    gather_forum_posts(&query, ctx, box_to_ref!(params))
}

pub fn get_forumposts_replies(
    ctx: &PageContext,
    root_post_id: i64,
) -> Result<Vec<ForumPost>, Error> {
    let (mut query, mut params) = forum_posts_base_query();
    query.push_str(
        " AND m.id IN (SELECT messageId FROM message_values WHERE `key`=? AND `value`=?)",
    );
    //params.push(Box::new(root_post_id));
    params.push(Box::new("re-top"));
    params.push(Box::new(root_post_id));
    gather_forum_posts(&query, ctx, box_to_ref!(params))
}

// fn get_forumpost_by_ids(ctx: &PageContext, id: i64) -> Result<Option<ForumPost>, Error> {
//     let (mut query, mut params) = forum_posts_base_query();
//     query.push_str(" AND m.id = ?");
//     params.push(Box::new(id));
//     Ok(gather_forum_posts(&query, ctx, box_to_ref!(params))?.pop())
// }

fn get_count_before_forumpost(ctx: &PageContext, id: i64) -> Result<i32, Error> {
    let query = format!(
        "SELECT COUNT(*) FROM messages m WHERE {} AND m.id < ? AND m.contentId = (SELECT contentId FROM messages WHERE id = ?)",
        COMMONMESSAGE
    );
    let mut stmt = ctx.dbcon.prepare(&query)?;
    let mut result = easy_query((&mut stmt, &query), rusqlite::params![id, id], |row| {
        row.get::<usize, i32>(0)
    })?;
    result.pop().ok_or(Error::NotFound(String::from(
        "PROGRAM ERROR: No results on count(*)??",
    )))
}

fn get_generic_message_request(
    query: &str,
    extra_uids: Vec<i64>,
    limit: i32,
    skip: i32,
) -> FullRequest {
    let mut request = FullRequest::new();
    add_value!(request, "uids", extra_uids);

    let message_request = build_request!(
        RequestType::message,
        String::from("*"),
        format!("!basiccomments() and ({})", query),
        String::from("id"),
        limit,
        skip
    );
    request.requests.push(message_request);

    let mut related_request = build_request!(
        RequestType::message,
        String::from("*"),
        String::from("!basiccomments() and id in @message.values.re"),
        String::from("id")
    );
    related_request.name = Some(String::from("related"));
    request.requests.push(related_request);

    //users in messages OR in extra_uids
    let user_request = build_request!(
        RequestType::user,
        String::from("*"),
        String::from(
            "id in @message.createUserId or id in @message.editUserId or \
                      id in @related.createUserId or id in @related.editUserId or id in @uids"
        )
    );
    request.requests.push(user_request);

    request
}

// //Apparently can't decide on transfered ownership or not
// pub fn get_finishpost_request(
//     thread_id: i64,
//     extra_uids: Vec<i64>,
//     limit: i32,
//     skip: i32,
// ) -> FullRequest {
//     let mut request =
//         get_generic_message_request("contentId = @thread_id", extra_uids, limit, skip);
//     add_value!(request, "thread_id", thread_id);
//     request
// }

pub fn get_thumbnail_hash2(content: &ForumThread2) -> Option<String> {
    if let Some(ref images) = get_value_safe!(&content.values, SBSValue::IMAGES, Vec<String>) {
        //values.get(SBSValue::IMAGES).and_then(|k| k.as_array()) {
        if let Some(image) = images.get(0) {
            return Some(image.clone());
        }
    }

    None
}

#[derive(Clone, Debug)]
pub struct ReplyData {
    pub top: i64,
    pub direct: i64,
}

// impl ReplyData {
//     pub fn write_to_values(&self, values: &mut HashMap<String, serde_json::Value>) {
//         values.insert(String::from("re-top"), self.top.into());
//         values.insert(String::from("re"), self.direct.into());
//     }
// }

/// Compute the flattened reply data for the given message
pub fn get_replydata(post: &ForumPost) -> Option<ReplyData> {
    if let Some(top) = get_value_safe!(post.values, "re-top", i64) {
        if let Some(direct) = get_value_safe!(post.values, "re", i64) {
            return Some(ReplyData { top, direct });
        }
    }
    return None;
}

/// Given a post, regenerate the new reply data that would properly point to this post
// pub fn get_new_replydata(post: &Message) -> ReplyData {
//     let id = post.id.unwrap_or_default();
//
//     let mut reply_data = ReplyData {
//         top: id,
//         direct: id,
//     };
//
//     //Oh but if we can parse existing reply data off the message, that one's top becomes our top too
//     if let Some(existing) = get_replydata(post) {
//         reply_data.top = existing.top;
//     }
//
//     reply_data
// }

#[derive(Clone, Debug)]
pub struct ReplyTree<'a> {
    pub id: i64,
    pub post: &'a ForumPost,
    pub children: Vec<ReplyTree<'a>>,
}

impl<'a> ReplyTree<'a> {
    pub fn new(message: &'a ForumPost) -> Self {
        ReplyTree {
            id: message.id,
            post: message,
            children: Vec::new(),
        }
    }

    /// Insert the given post as a node in this tree. Modifies the tree, and returns the node (if it was inserted)
    pub fn insert_post(&mut self, post: &'a ForumPost, data: &ReplyData) -> Option<&ReplyTree> {
        //If this is the node to insert into, return ourselves
        if self.id == data.direct {
            self.children.push(ReplyTree::new(post));
            return self.children.last();
        } else {
            //Otherwise, look through all the children. This is depth first recursion... whatever I guess.
            for node in self.children.iter_mut() {
                let result = node.insert_post(post, data);
                if result.is_some() {
                    return result;
                }
            }
        }

        //If we make it here, it was never found
        return None;
    }
}

/// Convert a list of posts into a tree. ASSUMES THE FIRST POST IS THE ROOT!!
pub fn posts_to_replytree(posts: &Vec<ForumPost>) -> Vec<ReplyTree> {
    if posts.len() == 0 {
        return Vec::new();
    }

    let mut root = ReplyTree::new(&posts[0]);

    for post in posts.iter().skip(1) {
        if let Some(data) = get_replydata(post) {
            if root.insert_post(post, &data).is_none() {
                println!(
                    "WARN: could not find place for message {}, reply to {}",
                    post.id, data.direct
                );
            }
        }
    }

    //println!("{:#?}", root);
    vec![root]
}

fn walk_post_tree(
    layout_data: &MainLayoutData,
    bbcode: &mut BBCode,
    config: &PostsConfig,
    tree: &ReplyTree,
    sequence: Option<i32>,
    posts_left: &mut i32,
) -> Markup {
    *posts_left -= 1;
    html! {
        //@let (sequence = config.start_num.and_then(|s| Some(s + index as i32));
        (post_item(layout_data, bbcode, config, tree.post, sequence))
        @if *posts_left > 0 { hr."smaller"; }
        @if tree.children.len() > 0 {
            div."replychain" {
                @for child in &tree.children {
                    //Note: only the very top level should get sequence numbers, so all inner recursive calls get None sequence
                    //@let (markup, posts_left) = (walk_post_tree(layout_data, &mut bbcode, config, child, None, posts_left - 1))
                    (walk_post_tree(layout_data, bbcode, config, child, None, posts_left))
                }
            }
        }
    }
}

/// Render the main sections of a content and message stream (the MAIN view on the website!) but configured
/// for the particular viewing instance. WARN: THIS ALSO MODIFIES context WITH APPROPRIATE OVERRIDES! A bit
/// more than rendering, I guess...
pub fn render_mainview(context: &mut PageContext, config: PostsConfig) -> Markup {
    let thread = &config.thread;
    let thread_type = &thread.literal_type; //.as_deref();

    let is_pagetype = thread_type == SBSPageType::PROGRAM
        || thread_type == SBSPageType::RESOURCE
        || thread_type == SBSPageType::DOCUMENTATION;

    //Here, we choose how the override works
    if thread_type == SBSPageType::PROGRAM || thread_type == SBSPageType::RESOURCE {
        context.layout_data.override_nav_path = Some("/search");
    } else if thread_type == SBSPageType::DOCUMENTATION {
        context.layout_data.override_nav_path = Some("/documentation");
    } else if thread_type == SBSPageType::DIRECTMESSAGE {
        context.layout_data.override_nav_path = Some("/userhome");
    }

    let data = &context.layout_data;
    let bbcode = &mut context.bbcode;
    let mut post_count = config.posts.len() as i32;

    let reply_tree: Vec<ReplyTree> = if config.render_reply_chain {
        posts_to_replytree(&config.posts)
    } else {
        // no reply chain is just a simple list of whatever
        config.posts.iter().map(|m| ReplyTree::new(m)).collect()
    };

    let mut pagelist_html: Option<Markup> = None;
    if let Some(ref pages) = config.pages {
        if pages.len() > 1 {
            pagelist_html = Some(html! {
                div."smallseparate pagelist" {
                    @for page in pages {
                        a."current"[page.current] target="_top" href={(data.links.forum_thread_unsafe(&thread.hash))"?page="(page.page)"#thread-top"} { (page.text) }
                    }
                }
            })
        }
    }

    html! {
        (data.links.style("/forpage/forum.css"))
        (data.links.script("/forpage/forum.js"))
        @if config.render_header {
            section {
                h1 title=(thread.id) { (thread.name) }
                @if let Some(path) = &config.path {
                    (forum_path(&data.links, &path))
                }
                div."foruminfo smallseparate aside" {
                    (threadicon2(&data.links, &thread))
                    //Snail doesn't want the create user displayed on documentation
                    @if thread_type != SBSPageType::DOCUMENTATION {
                        span {
                            @if let Some(user) = config.users.get(&thread.create_user_id) {
                                a."flatlink" target="_top" href=(data.links.user_unsafe(&user.username)){ (user.username) }
                            }
                        }
                    }
                    span {
                        b { "Created: " }
                        time datetime=(dd(&thread.create_date)) { (timeago(&thread.create_date)) }
                    }
                    iframe."votes" src={(data.links.votewidget_unsafe(thread.id))}{}
                }
            }
        }
        @if config.render_page && is_pagetype {
            (render_page(&data, bbcode, &thread, &config.categories)) //, &config.docs_content))
        }
        //it says "thread-top" because it is: it's the beginning of the section that displays posts. After the
        //for loop, it then displays pages, which is on the bottom of the thread, so it might seem confusing.
        //maybe the id should be changed to an anchor, idr how to do that.
        section #"thread-top" data-selected=[config.selected_post_id] {
            @if data.user_config.toppagination_posts {
                @if let Some(ref pagelist) = pagelist_html {
                    (pagelist)
                    hr."smaller";
                }
            }
            @if reply_tree.len() > 0 {
                @for (index,tree) in reply_tree.iter().enumerate() {
                    @let sequence = config.start_num.and_then(|s| Some(s + index as i32));
                    (walk_post_tree(&context.layout_data, &mut context.bbcode, &config, tree, sequence, &mut post_count))
                }
            }
            @else {
                //As usual, I'm reusing pagelist to make centered and spaced content
                p."aside pagelist" { "No posts yet (will you be the first?)" }
            }
            @if let Some(ref pagelist) = pagelist_html {
                (pagelist)
            }
        }
    }
}

//WAS consuming bbcode, now i'm not sure. leaving for now
pub fn post_item(
    layout_data: &MainLayoutData,
    bbcode: &mut BBCode,
    config: &PostsConfig,
    post: &ForumPost,
    sequence: Option<i32>,
) -> Markup {
    let users = &config.users;
    let user = user_or_default2(users.get(&post.create_user_id));
    let mut class = String::from("post");
    if config.selected_post_id == Some(post.id) {
        class.push_str(" current")
    }
    let mut reply_chain_link: Option<String> = None;
    let mut reply_post: Option<&ForumPost> = None;

    if let Some(replies) = get_replydata(post) {
        reply_post = config.related.get(&replies.direct);
        if reply_post.is_none() {
            println!("ERROR: couldn't find related post {}!", replies.direct)
        }
        if config.render_reply_link {
            let query = ThreadQuery {
                reply: Some(replies.top),
                selected: Some(post.id),
            };
            match serde_urlencoded::to_string(query) {
                Ok(query) => {
                    reply_chain_link = Some(format!(
                        "{}/widget/thread?{}",
                        &layout_data.links.http_root, query
                    ));
                }
                Err(error) => println!("ERROR: couldn't encode thread query!: {}", error),
            }
        }
    }

    html! {
        div.(class) #{"post_"(post.id)} {
            div."postleft" {
                img."avatar" src=(layout_data.links.image(&user.avatar, QueryImage::Cropped100 ));
            }
            div."postright" {
                div."postheader" {
                    a."flatlink username" target="_top" href=(layout_data.links.user_unsafe(&user.username)) { (&user.username) }
                    @if let Some(sequence) = sequence {
                        a."sequence" target="_top" title=(post.id) href=(layout_data.links.forum_post_unsafe(post.id, &config.thread.hash)){ "#" (sequence) }
                    }
                }
                @if let Some(reply_post) = reply_post {
                    //TODO: can't decide between consuming or not. spoilers are the important bit
                    (post_reply(layout_data, bbcode, reply_post, &config.thread.hash, &config.users))
                }
                div."content bbcode" data-postid=(post.id) { (PreEscaped(bbcode.parse(&post.text))) }
                div."postfooter mediumseparate" {
                    @if let Some(reply_link) = reply_chain_link {
                        details."repliesview aside" style="display:none" {
                            summary { "View conversation" }
                            iframe data-src=(reply_link){}
                        }
                    }
                    div."history" {
                        time."aside" datetime=(dd(&post.create_date)) { (timeago(&post.create_date)) }
                        @if let Some(edit_user_id) = post.edit_user_id {
                            time."aside" datetime=(d(&post.edit_date)) {
                                "Edited "(timeago_o(&post.edit_date))" by "
                                @if let Some(edit_user) = users.get(&edit_user_id) {
                                    a."flatlink" target="_top" href=(layout_data.links.user_unsafe(&edit_user.username)){ (&edit_user.username) }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn post_reply(
    layout_data: &MainLayoutData,
    bbcode: &mut BBCode,
    post: &ForumPost,
    thash: &str,
    users: &HashMap<i64, User2>,
) -> Markup {
    let user = user_or_default2(users.get(&post.create_user_id));
    html! {
        div."reply aside" {
            a."replylink" target="_top" href=(layout_data.links.forum_post_unsafe(post.id, thash)) { "Replying to:" }
            img src=(layout_data.links.image(&user.avatar, QueryImage::Cropped100 ));
            a."flatlink username" href=(layout_data.links.user_unsafe(&user.username)) { (&user.username) }
            div."content bbcode postpreview" { (PreEscaped(bbcode.parse(&post.text))) }
        }
    }
}

pub fn user_or_default(user: Option<&User>) -> User {
    if let Some(u) = user {
        u.clone()
    } else {
        User {
            username: String::from("???"),
            id: 0,
            avatar: String::from("0"),
            r#type: UserType::USER,
            special: None,
            createDate: chrono::Utc::now(),
            groups: Vec::new(),
        }
    }
}

// pub fn map_users(users: Vec<User>) -> HashMap<i64, User> {
//     users
//         .into_iter()
//         .map(|u| (u.id, u))
//         .collect::<HashMap<i64, User>>()
// }
/// Convert a vector of messages into a hashmap (id is key)
// pub fn map_messages(messages: Vec<Message>) -> HashMap<i64, Message> {
//     messages
//         .into_iter()
//         .map(|u| (u.id.unwrap_or_else(|| 0), u))
//         .collect::<HashMap<i64, Message>>()
// }
pub fn map_forumposts(posts: Vec<ForumPost>) -> HashMap<i64, ForumPost> {
    posts
        .into_iter()
        .map(|u| (u.id, u))
        .collect::<HashMap<i64, ForumPost>>()
}

#[derive(Deserialize, Debug)]
pub struct SpecialCount {
    pub specialCount: i32,
}

// --------------------------------------------
//             NEXT
// --------------------------------------------

pub fn render(mut context: PageContext, config: PostsConfig) -> String {
    let mut meta = LayoutMeta {
        title: format!("SBS ⦁ {}", config.thread.name),
        description: short_description2(&config.thread),
        image: get_thumbnail_hash2(&config.thread)
            .and_then(|h| Some(context.layout_data.links.image(&h, QueryImage::Scaled300))),
        canonical: Some(
            context
                .layout_data
                .links
                .forum_thread_unsafe(&config.thread.hash),
        ),
    };

    //If the literal type is a forumthread, we generally want to pull text from posts
    if config.thread.literal_type == SBSPageType::FORUMTHREAD {
        if let Some(selected_id) = config.selected_post_id {
            if let Some(post) = config.posts.iter().find(|p| p.id == selected_id) {
                meta.description = short_post(post);
                meta.canonical = Some(
                    context
                        .layout_data
                        .links
                        .forum_post_unsafe(post.id, &config.thread.hash),
                );
            }
        } else if let Some(start) = config.start_num {
            if start == 1 {
                //We KNOW this is the first page, so we can actually give the post as the description!
                if let Some(post) = config.posts.get(0) {
                    meta.description = short_post(post);
                }
            }
        }
    }

    let main_page = render_mainview(&mut context, config);
    layout_with_meta(&context.layout_data, meta, main_page).into_string()
}

pub fn get_thread_by_hash(ctx: &PageContext, hash: &str) -> Result<Option<ForumThread2>, Error> {
    let (mut query, mut params) = forum_theads_base_query();
    query.push_str(" AND t.hash = ?");
    params.push(Box::new(hash));
    Ok(gather_forum_threads(&query, ctx, box_to_ref!(params))?.pop())
}

pub fn get_forum_category_by_id(
    ctx: &PageContext,
    id: i64,
) -> Result<Option<ForumCategory2>, Error> {
    let (mut query, mut params) = forum_categories_base_query();
    query.push_str(" AND c.id = ?");
    params.push(Box::new(id));
    let mut stmt = ctx.dbcon.prepare(&query)?;
    let mut result = gather_forum_categories((&mut stmt, &query), box_to_ref!(params))?;
    Ok(result.pop())
}

fn add_uids_from_posts(messages: &Vec<ForumPost>, existing: &mut Vec<i64>) {
    for message in messages {
        existing.push(message.create_user_id);
        if let Some(edit_uid) = message.edit_user_id {
            existing.push(edit_uid);
        }
    }
}

async fn render_thread(
    context: PageContext,
    hash: &str,
    post_id: Option<i64>,
    per_page: i32,
    page: Option<i32>,
) -> Result<Response, Error> {
    let mut page = page.unwrap_or(1) - 1; //we assume 1-based pages

    let thread = get_thread_by_hash(&context, hash)?
        .ok_or(Error::NotFound(String::from("Could not find thread!")))?;
    let category = get_forum_category_by_id(&context, thread.parent_id)?
        .ok_or(Error::NotFound(String::from("Could not find category!")))?;
    let thread_subcat_ids = get_tagged_categories2(&thread.values);
    let subcategories = get_submission_categories(&context, Some(thread_subcat_ids))?;

    if let Some(post_id) = post_id {
        let precount = get_count_before_forumpost(&context, post_id)?;
        //The index is the special count. This means we change the page given. If page wasn't already 0, we warn
        if page != 0 {
            println!(
                "Page was nonzero ({}) while there was a message index ({})",
                page, precount
            );
        }
        page = precount / per_page;
    }

    //Also I need some fields to exist.
    let thread_id = thread.id;
    let thread_create_uid = thread.create_user_id;
    let comment_count = thread.posts_count;

    let sequence_start = page * per_page;

    let messages_raw = get_forumposts_basic(
        &context,
        thread_id,
        None,
        QueryLimit {
            limit: Some(per_page),
            skip: Some(sequence_start),
        },
    )?;
    let mut related_ids: Vec<i64> = Vec::new();
    for message in &messages_raw {
        if let Some(data) = get_replydata(message) {
            related_ids.push(data.direct);
        }
    }
    let related_raw = get_forumposts_basic(
        &context,
        thread_id,
        Some(related_ids),
        QueryLimit::default(),
    )?;
    let mut user_ids: Vec<i64> = vec![thread_create_uid];
    add_uids_from_posts(&messages_raw, &mut user_ids);
    add_uids_from_posts(&related_raw, &mut user_ids);
    let users_raw = get_users(&context, user_ids)?;

    //Construct before borrowing
    let path = vec![
        ForumPathItem::root(),
        ForumPathItem::from_category2(&category),
        ForumPathItem::from_thread2(&thread),
    ];
    let post_config = PostsConfig::thread_mode(
        thread,
        subcategories,
        messages_raw,
        map_forumposts(related_raw),
        map_users2(users_raw),
        path,
        get_pagelist(comment_count as i32, per_page, page),
        1 + per_page * page,
        post_id,
        //selected_post.and_then(|m| m.id),
    );
    Ok(Response::Render(render(context, post_config)))
}

/// The normal endpoint for listing a thread
pub async fn get_hash_render(
    context: PageContext,
    hash: String,
    per_page: i32,
    page: Option<i32>,
) -> Result<Response, Error> {
    let hurgh = hash.clone();
    render_thread(context, &hurgh, None, per_page, page).await
}

/// The normal endpoint for pinpointing a post
pub async fn get_hash_postid_render(
    context: PageContext,
    hash: String,
    post_id: i64,
    per_page: i32,
) -> Result<Response, Error> {
    let hurgh = hash.clone();
    render_thread(context, &hurgh, Some(post_id), per_page, None).await
}

// -------------------------------------
//          Widget thread
// -------------------------------------

/// Rendering for the actual widget. The
pub fn render_widget(context: &mut PageContext, config: PostsConfig) -> String {
    let posts = render_mainview(context, config);
    basic_skeleton(
        &context.layout_data,
        html! {
            title { "SmileBASIC Source Thread Widget" }
            meta name="description" content="Simple view into a thread";
            (context.layout_data.links.style("/forpage/forum.css"))
            style { r#"
            body { 
                /* This shrinks the WHOLE page! */
                font-size: 0.85rem; 
                padding: var(--space_medium);
            }
            @media screen and (max-width: 30em) {
                font-size: 0.75rem; 
            }
        "# }
        },
        html! {
            (posts)
        },
    )
    .into_string()
}

fn get_thread_by_msgid(ctx: &PageContext, msgid: i64) -> Result<Option<ForumThread2>, Error> {
    let (mut query, mut params) = forum_theads_base_query();
    query.push_str(" AND t.id = (SELECT m.contentId FROM messages m WHERE m.id = ?)");
    params.push(Box::new(msgid));
    Ok(gather_forum_threads(&query, ctx, box_to_ref!(params))?.pop())
}

pub async fn get_render_widget(
    mut context: PageContext,
    query: ThreadQuery,
) -> Result<Response, Error> {
    if let Some(post_id) = query.reply {
        let thread = get_thread_by_msgid(&context, post_id)?
            .ok_or(Error::NotFound(String::from("Could not find thread!")))?;

        //let api_context = ApiContext::new(String::from("http://localhost:5000/api"));

        let messages_raw = get_forumposts_basic(
            &context,
            thread.id,
            Some(vec![post_id]),
            QueryLimit::default(),
        )?;
        let related_raw = get_forumposts_replies(&context, post_id)?;
        let mut user_ids: Vec<i64> = Vec::new();
        add_uids_from_posts(&messages_raw, &mut user_ids);
        add_uids_from_posts(&related_raw, &mut user_ids);
        let users_raw = get_users(&context, user_ids)?;

        Ok(Response::Render(render_widget(
            &mut context,
            PostsConfig::reply_mode(
                thread,
                messages_raw,
                map_forumposts(related_raw),
                map_users2(users_raw),
                query.selected,
            ),
        )))
    } else {
        Err(Error::Other(String::from(
            "No data provided; this widget requires at least 'reply'",
        )))
    }
}
