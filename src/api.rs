use std::fmt;
use std::io::Read;

use rocket::serde::DeserializeOwned;
use serde::Serialize;
use crate::context::Context;
use crate::forms;
use crate::api_data::*;
use thiserror::Error;

//These are the specific types of errors we'll care about from the api
#[derive(Error, Debug)]
pub enum ApiError
{
    #[error("Pre-API Preparation Error: {0}")]
    Precondition(String),   //Something BEFORE the api failed (some kind of setup?)
    #[error("Network Error: {0}")]
    Network(String),    //Is the API reachable?
    #[error("API Usage Error: {0}")]
    Usage(String),      //Did I (the programmer) use it correctly?
    #[error("Request Error[{0}]: {1}")]
    User(RequestStatus, String) //Did the user submit proper data?
}

#[derive(Debug)]
pub enum RequestStatus {
    Reqwest(reqwest::StatusCode),
    None
}


impl fmt::Display for RequestStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Reqwest(status) => write!(f, "{}", status),
            Self::None => write!(f, "???")
        }
    }
}

impl From<Option<reqwest::StatusCode>> for RequestStatus {
    fn from(item: Option<reqwest::StatusCode>) -> Self {
        match item {
            Some(status) => RequestStatus::Reqwest(status),
            None => RequestStatus::None
        }
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
    ($endpoint:expr, $status:expr, $data:expr) => {
       |err| ApiError::Usage(String::from(format!("Could not parse RESPONSE body from {}[{}], serde error: {}\nREQUEST DATA:\n{:?}", $endpoint, $status, err, $data)))
    };
}


//Once a response comes back from the API, figure out the appropriate errors or data to parse and return
macro_rules! handle_response {
    ($response:ident, $endpoint:expr, $data:expr) => 
    {
        //Another block so we can copy some values out of the response
        {
            let status = $response.status();
            match $response.error_for_status_ref()
            {
                //The result from the API was fine, try to parse it as json
                Ok(_) => {
                    $response.json::<T>().await.map_err(parse_error!($endpoint, status, $data))
                },
                //The result from the API was 400, 500, etc. Try to parse the body as the error
                Err(response_error) => {
                    match $response.text().await.map_err(parse_error!($endpoint, status, $data)) {
                        Ok(real_error) => Err(ApiError::User(response_error.status().into(), real_error)),
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
    
    handle_response!(response, endpoint, "GET REQUEST (empty)")
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
    where U : Serialize+core::fmt::Debug, T: DeserializeOwned
{
    let request = create_post_request(endpoint, context);

    //See above for info on why mapping request errors to string is fine
    let response = request
        .header("Content-Type","application/json")
        .json(data)
        .send().await
        .map_err(network_error!())?;
    
    handle_response!(response, endpoint, data)
}

pub async fn get_about(context: &Context) -> Result<About, ApiError>
{
    basic_get_request("/status", context).await
}

//This consumes the error and returns "None", since it could just be that the token is stupid. In the future,
//we may want to alert the user that their token is invalid somewhere, which would require propogating the
//error result AND checking the status to determine if we need JSON or not...
pub async fn get_user_safe(context: &Context) -> Option<User>
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

pub async fn get_user_private_safe(context: &Context) -> Option<UserPrivate>
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

pub async fn post_userupdate<'a>(context: &Context, user: &User) -> Result<User, ApiError>
{
    basic_post_request("/write/user", user, context).await
}

pub async fn upload_file<'a>(context: &Context, form: &mut forms::FileUpload<'_>) -> Result<Content, ApiError>
{
    //println!("Received form: {:?}, length: {}", form, form.file.len());

    //First step is to get a temporary path. 
    let named_file = tempfile::NamedTempFile::new().map_err(precondition_error!())?;
    let temp_path = named_file.into_temp_path(); //When this goes out of scope, the file is supposedly deleted. So DON'T SHADOW IT

    //Now, persist the uploaded file to the path. Remember, temp_path needs to be persisted, so don't transfer ownership!
    form.file.persist_to(&temp_path).await.map_err(precondition_error!())?;

    let mut content = Content::default();
    content.contentType = Some(ContentType::FILE);

    //This is the data we're uploading. We'll be filling in the base64 next
    let mut data = FileUploadAsObject {
        base64blob : String::new(), 
        object : content//Content::new(ContentType::FILE) 
    };
    
    //OK now that it's on the filesystem, gotta read it as base64
    let file = std::fs::File::open(&temp_path).map_err(precondition_error!())?;
    let mut b64reader = base64_stream::ToBase64Reader::new(&file);
    b64reader.read_to_string(&mut data.base64blob).map_err(precondition_error!())?;

    let result = basic_post_request("/file/asobject", &data, context).await;

    //This ensures the compiler will complain if temp_path goes out of scope
    if let Err(error) = temp_path.close() {
        println!("Couldn't delete temporary file (this is ok): {}", error);
    }
    
    result

}