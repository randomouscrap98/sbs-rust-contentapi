use std::fmt;

use rocket::{http::Status, serde::DeserializeOwned};
use serde::{Serialize, Deserialize};
use crate::context::Context;
use crate::forms;

use rocket::response::status::Custom as RocketCustom;

pub enum ApiError
{
    Network(String),
    Usage(String),
    User(Option<reqwest::StatusCode>, String)
}

impl ApiError
{
    pub fn get_to_string(&self) -> String {
        match self {
            ApiError::Network(s) => format!("Network Error: {}", s),
            ApiError::Usage(s) => format!("API Usage Error: {}", s),
            ApiError::User(_, s) => format!("Request Error: {}", s),
        }
    }

    //No default params makes me sad
    pub fn get_just_string(&self) -> String {
        match self {
            ApiError::Network(s) => format!("{}", s),
            ApiError::Usage(s) => format!("{}", s),
            ApiError::User(_, s) => format!("{}", s),
        }
    }
}

impl fmt::Display for ApiError 
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_to_string())
    }
}

//Generate the simple closure for map_error when reading the body of a response. These should not
//actually happen and indicate an error with the API, so I'm fine just outputting a general
//error with additional info.
macro_rules! parse_error {
    ($endpoint:expr) => {
       |err| ApiError::Usage(String::from(format!("Could not parse result/body from {}: response error: {}!", $endpoint, err)))
    };
}

macro_rules! network_error {
    () => {
       |err| ApiError::Network(String::from(format!("Server unavailable: {}", err)))
    };
}

//Simple conversion from server error into rocket error. This should ONLY be used where we are certain
//the error isn't due to the user!
macro_rules! rocket_error {
    () => {
        |e| RocketCustom(Status::ServiceUnavailable, e.to_string())
    };
}

macro_rules! handle_response {
    ($response:ident, $endpoint:expr) => 
    {
        match $response.error_for_status_ref()
        {
            //The result from the API was fine, try to parse it as json
            Ok(_) => {
                $response.json::<T>().await.map_err(parse_error!($endpoint))
            },
            //The result from the API was 400, 500, etc. Try to parse the body as the error
            Err(response_error) => {
                match $response.json::<String>().await.map_err(parse_error!($endpoint)) {
                    Ok(real_error) => Err(ApiError::User(response_error.status(), real_error)),
                    Err(p_error) => Err(p_error)
                }
            }
        }
    };
}

#[derive(Serialize, Deserialize)]
pub struct About
{
    version: String,
    environment: String,
    runtime: String,
    contact: String
}

#[derive(Serialize, Deserialize)]
pub struct User
{
    id: u64,
    username: String,
    avatar: String
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

//Construct a basic POST request to the given endpoint (including ?params) using the given
//request context. Automatically add bearer headers and all that
pub async fn basic_post_request<U, T>(endpoint: &str, data: &U, context: &Context) -> Result<T, ApiError>
    where U : Serialize, T: DeserializeOwned
{
    let mut request = context.client.post(context.config.get_endpoint(endpoint));
    
    if let Some(token) = &context.user_token {
        request = request.bearer_auth(token);
    }

    //See above for info on why mapping request errors to string is fine
    let response = request
        .header("Accept", "application/json")
        .header("Content-Type","application/json")
        .json(data)
        .send().await
        .map_err(network_error!())?;
    
    handle_response!(response, endpoint)
}

pub async fn get_about_rocket(context: &Context) -> Result<About, RocketCustom<String>> 
{
    basic_get_request("/status", context).await.map_err(rocket_error!())
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

//Not a rocket version because we want the errors from the API
pub async fn post_login<'a>(context: &Context, login: &forms::Login<'a>) -> Result<String, ApiError>
{
    basic_post_request::<forms::Login<'a>, String>("/user/login", login, context).await
}