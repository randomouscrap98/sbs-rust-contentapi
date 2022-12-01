use std::fmt;

use rocket::serde::DeserializeOwned;
use serde::Serialize;
use tokio_util::codec::BytesCodec;
use tokio_util::codec::FramedRead;
use crate::context::Context;
use crate::forms;
use crate::api_data::*;

//These are the specific types of errors we'll care about from the api
pub enum ApiError
{
    Precondition(String),   //Something BEFORE the api failed (some kind of setup?)
    Network(String),    //Is the API reachable?
    Usage(String),      //Did I (the programmer) use it correctly?
    User(Option<reqwest::StatusCode>, String) //Did the user submit proper data?
}

//impl ApiError
//{
//    //This is the implementation of "display". Since I have multiple display formats,
//    //I wanted there to be consistent functions.
//    pub fn get_to_string(&self) -> String {
//    }
//
//    ////No default params makes me sad
//    //pub fn get_just_string(&self) -> String {
//    //    match self {
//    //        ApiError::Network(s) => format!("{}", s),
//    //        ApiError::Usage(s) => format!("{}", s),
//    //        ApiError::User(_, s) => format!("{}", s),
//    //    }
//    //}
//}

impl fmt::Display for ApiError 
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            ApiError::Precondition(s) => format!("Pre-API Preparation Error: {}", s),
            ApiError::Network(s) => format!("Network Error: {}", s),
            ApiError::Usage(s) => format!("API Usage Error: {}", s),
            ApiError::User(_, s) => format!("Request Error: {}", s),
        })
        //write!(f, "{}", self.get_to_string())
    }
}

macro_rules! precondition_error {
    () => {
       |err| ApiError::Precondition(err.to_string()) //String::from(format!("Pre-API preparation failed: {}", err)))
    };
}

//Generate the simple closure for map_error for when the API is unreachable
macro_rules! network_error {
    () => {
       |err| ApiError::Network(err.to_string()) //String::from(format!("Server unavailable: {}", err)))
    };
}

//Generate the simple closure for map_error when reading the body of a response. These should not
//actually happen and indicate an error with the API, so I'm fine just outputting a general
//error with additional info.
macro_rules! parse_error {
    ($endpoint:expr, $status:expr) => {
       |err| ApiError::Usage(String::from(format!("Could not parse result/body from {}({}): response error: {}!", $endpoint, $status, err)))
    };
}


//Once a response comes back from the API, figure out the appropriate errors or data to parse and return
macro_rules! handle_response {
    ($response:ident, $endpoint:expr) => 
    {
        //Another block so we can copy some values out of the response
        {
            let status = $response.status();
            match $response.error_for_status_ref()
            {
                //The result from the API was fine, try to parse it as json
                Ok(_) => {
                    $response.json::<T>().await.map_err(parse_error!($endpoint, status))
                },
                //The result from the API was 400, 500, etc. Try to parse the body as the error
                Err(response_error) => {
                    match $response.text().await.map_err(parse_error!($endpoint, status)) {
                        Ok(real_error) => Err(ApiError::User(response_error.status(), real_error)),
                        Err(p_error) => Err(p_error)
                    }
                }
            }
        }
    };
}


//Construct a basic GET request to the given endpoint (including ?params) using the given
//request context. Automatically add bearer headers and all that. Errors on the appropriate
//status codes, message is assumed to be parsed from body
pub async fn basic_get_request<T>(endpoint: &str, context: &Context) -> Result<T, ApiError>
    where T: DeserializeOwned
{
    let mut request = context.client.get(context.config.get_endpoint(endpoint));
    
    if let Some(token) = &context.user_token {
        request = request.bearer_auth(token);
    }

    //Mapping the request error to a string is PERFECTLY ok in this library because these errors are
    //NOT from stuff like 400 or 500 statuses, they're JUST from network errors (it's localhost so
    //it should never happen, and I'm fine with funky output for the few times there are downtimes)
    let response = request
        .header("Accept", "application/json")
        .send().await
        .map_err(network_error!())?;
    
    handle_response!(response, endpoint)
}

// The basis for creating POST requests (since we have a couple types)
fn create_post_request(endpoint: &str, context: &Context) -> reqwest::RequestBuilder {
    let mut request = context.client.post(context.config.get_endpoint(endpoint))
        .header("Accept", "application/json");
    
    if let Some(token) = &context.user_token {
        request = request.bearer_auth(token);
    }

    request
}

//Construct a basic POST request to the given endpoint (including ?params) using the given
//request context. Automatically add bearer headers and all that
pub async fn basic_post_request<U, T>(endpoint: &str, data: &U, context: &Context) -> Result<T, ApiError>
    where U : Serialize, T: DeserializeOwned
{
    let request = create_post_request(endpoint, context);

    //See above for info on why mapping request errors to string is fine
    let response = request
        .header("Content-Type","application/json")
        .json(data)
        .send().await
        .map_err(network_error!())?;
    
    handle_response!(response, endpoint)
}

pub async fn basic_upload_request<T: DeserializeOwned>(endpoint: &str, data: reqwest::multipart::Form, context: &Context) -> Result<T, ApiError>
{
    let request = create_post_request(endpoint, context);
    let response = request
        .multipart(data)
        .send().await
        .map_err(network_error!())?;

    handle_response!(response, endpoint)
}
    //where T: DeserializeOwned

pub async fn get_about(context: &Context) -> Result<About, ApiError>//RocketCustom<String>> 
{
    basic_get_request("/status", context).await//.map_err(rocket_error!())
}

//This consumes the error and returns "None", since it could just be that the token is stupid. In the future,
//we may want to alert the user that their token is invalid somewhere, which would require propogating the
//error result AND checking the status to determine if we need JSON or not...
pub async fn get_user_safe(context: &Context) -> Option<User> //Result<Option<User>, RocketCustom<String>>
{
    //Only run if there IS a token
    if let Some(_) = context.user_token 
    {
        //Once we have the token, try it against the api. If there's an error, just print it and move on
        //with apparently "no" user
        match basic_get_request::<User>("/user/me", context).await 
        {
            Ok(result) => Some(result),
            Err(error) => { 
                println!("User token error: {}", error); 
                None
            }
        }
    }
    else
    {
        None
    }
}

pub async fn get_user_private_safe(context: &Context) -> Option<UserPrivate> //Result<Option<User>, RocketCustom<String>>
{
    //Only run if there IS a token
    if let Some(_) = context.user_token 
    {
        //Once we have the token, try it against the api. If there's an error, just print it and move on
        //with apparently "no" user
        match basic_get_request::<UserPrivate>("/user/privatedata", context).await 
        {
            Ok(result) => Some(result),
            Err(_error) => None,
        }
    }
    else
    {
        None
    }
}

pub async fn post_request(context: &Context, request: &FullRequest) -> Result<RequestResult, ApiError>
{
    basic_post_request("/request", request, context).await
}

//Not a rocket version because we want the errors from the API
pub async fn post_login<'a>(context: &Context, login: &Login) -> Result<String, ApiError>
{
    basic_post_request("/user/login", login, context).await
}

pub async fn post_register<'a>(context: &Context, register: &forms::Register<'a>) -> Result<User, ApiError>
{
    basic_post_request("/user/register", register, context).await
}

pub async fn post_email_confirm<'a>(context: &Context, email: &str) -> Result<bool, ApiError>
{
    basic_post_request("/user/sendregistrationcode", &email, context).await
}

pub async fn post_email_recover<'a>(context: &Context, email: &str) -> Result<bool, ApiError>
{
    basic_post_request("/user/sendpasswordrecovery", &email, context).await
}

pub async fn post_registerconfirm<'a>(context: &Context, confirm: &forms::RegisterConfirm<'a>) -> Result<String, ApiError>
{
    basic_post_request("/user/confirmregistration", confirm, context).await
}

pub async fn post_usersensitive<'a>(context: &Context, sensitive: &forms::UserSensitive<'_>) -> Result<bool, ApiError>
{
    basic_post_request("/user/privatedata", sensitive, context).await
}

pub async fn create_basic_multipart_part(path: &std::path::Path) -> Result<reqwest::multipart::Part, ApiError>
{
    let file = tokio::fs::File::open(path).await.map_err(precondition_error!())?;
    let stream = FramedRead::new(file, BytesCodec::new());
    let file_body = reqwest::Body::wrap_stream(stream);

    //I don't think the API uses ANY of the "filename" "mimetype" stuff
    Ok(reqwest::multipart::Part::stream(file_body)) 
}

pub async fn upload_file<'a>(context: &Context, form: &forms::FileUpload<'_>) -> Result<Content, ApiError>
{
    let path = form.file.path().ok_or(ApiError::Precondition(String::from("Path could not be retrieved from TempFile")))?;
    let form = reqwest::multipart::Form::new()
        //.text("", "")
        .part("file", create_basic_multipart_part(&path).await?);

    //let some_file = reqwest::multipart::Part::stream(file_body)
    //    .file_name("gitignore.txt")
    //    .mime_str("text/plain")?;
    
    basic_upload_request("/file", form, context).await
}