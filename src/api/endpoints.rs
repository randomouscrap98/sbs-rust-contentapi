use std::fmt;
//use std::io::Read;

use reqwest::Client;
use serde::Serialize;
use serde::de::DeserializeOwned;
//use thiserror::Error;

use forms;
use super::*;

//There is some "context" that represents a current user and their client connection,
//as well as the api endpoint to connect to. This is used to craft requests on your behalf
pub trait Context {
    fn get_api_url(&self) -> &str;
    fn get_client(&self) -> Client;
    fn get_user_token(&self) -> Option<&str>;

    fn get_endpoint(&self, endpoint: &str) -> String {
        format!("{}{}", self.get_api_url(), endpoint)
    }
}

//These are the specific types of errors we'll care about from the api
//#[derive(Error, Debug)]
#[derive(Debug)]
pub enum ApiError
{
    //#[error("Pre-API Preparation Error: {0}")]
    Precondition(String),   //Something BEFORE the api failed (some kind of setup?)
    //#[error("Network Error: {0}")]
    Network(String),    //Is the API reachable?
    //#[error("API Usage Error: {0}")]
    Usage(String, String),      //Did I (the programmer) use it correctly? Also pass the data, DON'T display that!
    //#[error("Request Error[{0}]: {1}")]
    User(RequestStatus, String, String) //Did the user submit proper data? Also pass data (last param), again DON'T DISPLAY
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
       |err| ApiError::Usage(String::from(format!("Could not parse RESPONSE body from {}[{}], serde error: {}", $endpoint, $status, err)), format!("{:?}", $data))
       //|err| ApiError::Usage(String::from(format!("Could not parse RESPONSE body from {}[{}], serde error: {}\nREQUEST DATA:\n{:?}", $endpoint, $status, err, $data)))
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
                    //Note: we map the error preemptively to let us use the macro
                    match $response.text().await.map_err(parse_error!($endpoint, status, $data)) {
                        Ok(api_text_error) => Err(ApiError::User(response_error.status().into(), format!("At endpoint '{}': {}", $endpoint, api_text_error), format!("{:?}", $data))),
                        //Ok(api_text_error) => Err(ApiError::User(response_error.status().into(), format!("At endpoint '{}': {}\nREQUEST DATA:\n{:?}", $endpoint, api_text_error, $data))),
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
pub async fn basic_get_request<T>(endpoint: &str, context: &impl Context) -> Result<T, ApiError>
    where T: DeserializeOwned
{
    let mut request = context.get_client().get(context.get_endpoint(endpoint));
    
    if let Some(token) = &context.get_user_token() {
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
fn create_post_request(endpoint: &str, context: &impl Context) -> reqwest::RequestBuilder {
    let mut request = context.get_client().post(context.get_endpoint(endpoint))
        .header("Accept", "application/json");
    
    if let Some(token) = &context.get_user_token() {
        request = request.bearer_auth(token);
    }

    request
}

//Construct a basic POST request to the given endpoint (including ?params) using the given
//request context. Automatically add bearer headers and all that
pub async fn basic_post_request<U, T>(endpoint: &str, data: &U, context: &impl Context) -> Result<T, ApiError>
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

pub async fn get_about(context: &impl Context) -> Result<About, ApiError>
{
    basic_get_request("/status", context).await
}

//pub async fn get_generic_safe<T: DeserializeOwned>(context: &impl Context, endpoint: &str endpoint) -> Option<T>
//{
//    //Only run if there IS a token
//    if let Some(_) = context.get_user_token()
//    {
//        //Once we have the token, try it against the api. If there's an error, just print it and move on
//        //with apparently "no" user
//        match basic_get_request::<User>("/user/me", context).await 
//        {
//            Ok(result) => Some(result),
//            Err(error) => { 
//                println!("User token error: {}", error); 
//                None
//            }
//        }
//    }
//    else
//    {
//        None
//    }
//}

//This consumes the error and returns "None", since it could just be that the token is stupid. In the future,
//we may want to alert the user that their token is invalid somewhere, which would require propogating the
//error result AND checking the status to determine if we need JSON or not...
pub async fn get_user_safe(context: &impl Context) -> Option<User>
{
    //Only run if there IS a token
    if let Some(_) = context.get_user_token()
    {
        //Once we have the token, try it against the api. If there's an error, just print it and move on
        //with apparently "no" user
        match basic_get_request::<User>("/user/me", context).await 
        {
            Ok(result) => Some(result),
            Err(error) => { 
                //println!("User token error: {}", error); 
                None
            }
        }
    }
    else
    {
        None
    }
}

pub async fn get_user_private_safe(context: &impl Context) -> Option<UserPrivate>
{
    //Only run if there IS a token
    if let Some(_) = context.get_user_token()
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


//Not a rocket version because we want the errors from the API
pub async fn post_login<'a>(context: &impl Context, login: &forms::Login<'_>) -> Result<String, ApiError>
{
    basic_post_request("/user/login", login, context).await
}

pub async fn post_register<'a>(context: &impl Context, register: &forms::Register<'a>) -> Result<User, ApiError>
{
    basic_post_request("/user/register", register, context).await
}

pub async fn post_email_confirm(context: &impl Context, email: &forms::EmailGeneric<'_>) -> Result<bool, ApiError>
{
    basic_post_request("/user/sendregistrationcode", &email, context).await
}

pub async fn post_email_recover(context: &impl Context, email: &forms::EmailGeneric<'_>) -> Result<bool, ApiError>
{
    basic_post_request("/user/sendpasswordrecovery", &email, context).await
}

pub async fn post_registerconfirm(context: &impl Context, confirm: &forms::RegisterConfirm<'_>) -> Result<String, ApiError>
{
    basic_post_request("/user/confirmregistration", confirm, context).await
}

pub async fn post_usersensitive(context: &impl Context, sensitive: &forms::UserSensitive<'_>) -> Result<String, ApiError>
{
    basic_post_request("/user/privatedata", sensitive, context).await
}



pub async fn post_request(context: &impl Context, request: &FullRequest) -> Result<RequestResult, ApiError>
{
    basic_post_request("/request", request, context).await
}

pub async fn post_userupdate(context: &impl Context, user: &User) -> Result<User, ApiError>
{
    basic_post_request("/write/user", user, context).await
}

pub async fn post_content(context: &impl Context, content: &Content) -> Result<Content, ApiError>
{
    basic_post_request("/write/content", content, context).await
}

//pub async fn upload_file<'a>(context: &Context, form: &mut forms::FileUpload<'_>) -> Result<Content, ApiError>
//{
//    //println!("Received form: {:?}, length: {}", form, form.file.len());
//
//    //First step is to get a temporary path. 
//    let named_file = tempfile::NamedTempFile::new().map_err(precondition_error!())?;
//    let temp_path = named_file.into_temp_path(); //When this goes out of scope, the file is supposedly deleted. So DON'T SHADOW IT
//
//    //Now, persist the uploaded file to the path. Remember, temp_path needs to be persisted, so don't transfer ownership!
//    form.file.persist_to(&temp_path).await.map_err(precondition_error!())?;
//
//    let mut content = Content::default();
//    content.contentType = Some(ContentType::FILE);
//
//    //This is the data we're uploading. We'll be filling in the base64 next
//    let mut data = FileUploadAsObject {
//        base64blob : String::new(), 
//        object : content//Content::new(ContentType::FILE) 
//    };
//    
//    //OK now that it's on the filesystem, gotta read it as base64
//    let file = std::fs::File::open(&temp_path).map_err(precondition_error!())?;
//    let mut b64reader = base64_stream::ToBase64Reader::new(&file);
//    b64reader.read_to_string(&mut data.base64blob).map_err(precondition_error!())?;
//
//    let result = basic_post_request("/file/asobject", &data, context).await;
//
//    //This ensures the compiler will complain if temp_path goes out of scope
//    if let Err(error) = temp_path.close() {
//        println!("Couldn't delete temporary file (this is ok): {}", error);
//    }
//    
//    result
//
//}