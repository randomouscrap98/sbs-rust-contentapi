use core::fmt::Debug;

//use hyper::Uri;
//use hyper::client::HttpConnector;
//use hyper::client::HttpConnector;
//use reqwest::Client;
use serde::Serialize;
use serde::de::DeserializeOwned;

use forms;
use super::*;

//There is some "context" that represents a current user and their client connection,
//as well as the api endpoint to connect to. This is used to craft requests on your behalf


//These are the specific types of errors we'll care about from the api. In all instances, the String
//is a minimal amount of data to show the users. The rest is for logging
//#[derive(Error, Debug)]
#[derive(Debug)]
pub enum ApiError
{
    NonRequest(AboutRequest, String),   //Something not pertaining to the actual request itself happened!
    Parse(AboutRequest, String),        //Something didn't parse correctly! This is common enough to be its own error
    Network(AboutRequest, String),      //Is the API reachable? Endpoint not necessary most likely; this indicates an error beyond 404
    Request(AboutRequest, String, u16), //Oh something went wrong with the request itself! Probably a 400 or 500 error
}

#[derive(Debug, Clone)]
pub struct AboutRequest {
    //This is GET/POST/etc. I don't care for it to be an enum, since I'm just printing it
    pub verb: String,
    pub endpoint: String,
    //Restricted data, which should probably not even be logged to the console! So what do
    //we do with it? It's mostly just for debugging I think, there may be a flag to enable
    //printing the restricted data
    pub post_data: Option<String>
}

/// This is needed so often: just convert any generic error into a "no request" error,
/// assuming you have the AboutRequest...
macro_rules! noreqerr {
    ($result:expr, $req:ident) => {
        $result.map_err(|e| ApiError::NonRequest($req.clone(), e.to_string()))
    };
}
//pub(crate) use noreqerr;

/// This isn't needed as often: just convert any generic error into a "network" error
macro_rules! neterr {
    ($result:expr, $req:ident) => {
        $result.map_err(|e| ApiError::Network($req.clone(), e.to_string()))
    };
}

/// This isn't needed as often: just convert any generic error into a "parse" error
macro_rules! parseerr {
    ($result:expr, $req:ident) => {
        $result.map_err(|e| ApiError::Parse($req.clone(), e.to_string()))
    };
}

//You'll want to create a new api context to make multiple requests, as it's more efficient.
//Maybe one per request?
pub struct ApiContext {
    api_url: String,
    client: hyper::client::Client<hyper::client::HttpConnector>,
    //client: reqwest::Client,
    user_token: Option<String>
}

impl ApiContext {
    pub fn new(api_url: String, user_token: Option<String>) -> Self {
        Self {
            api_url, user_token,
            client : hyper::client::Client::new() //reqwest::Client::new()
        }
    }

    pub fn get_endpoint(&self, endpoint: &str) -> String {
        format!("{}{}", self.api_url, endpoint)
    }

    /// All requests to the API start off the same
    fn get_request_builder(&self, request: &AboutRequest, method: hyper::Method) -> Result<hyper::http::request::Builder, ApiError> 
    {
        let endpoint_uri = noreqerr!(self.get_endpoint(&request.endpoint).parse::<hyper::Uri>(), request)?;

        let mut reqbuilder = hyper::Request::builder()
            .method(method)
            .uri(endpoint_uri);
        
        if let Some(token) = &self.user_token {
            reqbuilder = reqbuilder.header("Authorization", format!("Bearer {}", token));
        }

        Ok(reqbuilder)
    }

    //Once a response comes back from the API, figure out the appropriate errors or data to parse and return
    async fn handle_response<T: DeserializeOwned>(response: hyper::Response<hyper::Body>, about: AboutRequest) -> Result<T, ApiError> {
        let status = response.status();
        let u_status = status.as_u16();

        let body = parseerr!(hyper::body::to_bytes(response.into_body()).await, about)?;
        //response.into_body()
        //let (parts, body) = response.into_parts();
        //let body = serde_json::from_slice(&body)?;

        //Good status vs all the rest.
        if status.is_success() {
            parseerr!(serde_json::from_slice::<T>(&body), about)
        }
        else {
            match String::from_utf8(body.into_iter().collect()) {
                Ok(error) => Err(ApiError::Request(about, error, u_status)),
                Err(error) => Err(ApiError::Request(about, format!("RESPONSE BODY UTF-8 ERROR: {}", error), u_status))
            }
            //Err(ApiError::Request(about, String::from_ut
        }
        //match response.error_for_status_ref()
        //{
        //    //The result from the API was fine, try to parse it as json (which might fail)
        //    Ok(_) => response.json::<T>().await.map_err(|e| ApiError::Parse(about, e.to_string())) ,
        //    //The result from the API was 400, 500, etc. Try to parse the body as the error
        //    Err(_) => Err(match response.text().await { //.map_err(parse_error!($endpoint, status, $data)) {
        //            Ok(api_text_error) => ApiError::Request(about, api_text_error, u_status), //response_error.status().into(), format!("At endpoint '{}': {}", $endpoint, api_text_error), format!("{:?}", $data))),
        //            //Ok(api_text_error) => Err(ApiError::User(response_error.status().into(), format!("At endpoint '{}': {}\nREQUEST DATA:\n{:?}", $endpoint, api_text_error, $data))),
        //            Err(p_error) => ApiError::NonRequest(about, p_error.to_string())
        //        })
        //}
    }

    //fn get_request_from_body(reqbuilder: hyper::http::request::Builder, body: hyper::Body) -> Result<hyper::Request<hyper::Body>, ApiError> {
    //    reqbuilder.body(body).map_err(|e| ApiError::NonRequest(request.clone(), e.to_string()))
        //) {
        //    Ok(request) => request,
        //    Err(error) => Err(ApiError::NonRequest(request.clone(), error.to_string()))
        //}
    //}

    //Construct a basic GET request to the given endpoint (including ?params) using the given
    //request context. Automatically add bearer headers and all that. Errors on the appropriate
    //status codes, message is assumed to be parsed from body
    pub async fn basic_get_request<T: DeserializeOwned>(&self, request: AboutRequest) -> Result<T, ApiError>
    {
        let reqbuilder = self.get_request_builder(&request, hyper::Method::GET)?
            .header("Accept", "application/json");
        let req = noreqerr!(reqbuilder.body(hyper::Body::empty()), request)?;

        //Mapping the request error to a string is PERFECTLY ok in this library because these errors are
        //NOT from stuff like 400 or 500 statuses, they're JUST from network errors (it's localhost so
        //it should never happen, and I'm fine with funky output for the few times there are downtimes)
        let response = neterr!(self.client.request(req).await, request)?;
            //.header("Accept", "application/json")).await;
            //.send().await
            //.map_err(|e| ApiError::Network(request.clone(), e.to_string()))?;
        
        Self::handle_response(response, request).await
    }

    // The basis for creating POST requests (since we have a couple types)
    //fn create_post_request(&self, endpoint: &str) -> reqwest::RequestBuilder {
    //    let mut request = self.client.post(self.get_endpoint(endpoint))
    //        .header("Accept", "application/json");
    //    
    //    if let Some(token) = &self.user_token {
    //        request = request.bearer_auth(token);
    //    }

    //    request
    //}

    //Construct a basic POST request to the given endpoint (including ?params) using the given
    //request context. Automatically add bearer headers and all that
    pub async fn basic_post_request<U: Serialize+Debug, T: DeserializeOwned>(&self, request: AboutRequest, data: &U) -> Result<T, ApiError>
    {
        let reqbuilder = self.get_request_builder(&request, hyper::Method::GET)?
            .header("Content-Type", "application/json");
        let json = noreqerr!(serde_json::ser::to_string(data), request)?; //Even though this is serde, it's not a parse error because it's before the request
        let req = noreqerr!(reqbuilder.body(hyper::Body::from(json)), request)?; //.map_err(|e| ApiError::NonRequest(request.clone(), e.to_string()))?;
        //let client = self.create_post_request(&request.endpoint);

        let response = self.client.request(req).await
            .map_err(|e| ApiError::Network(request.clone(), e.to_string()))?;

        //See above for info on why mapping request errors to string is fine
        //let response = client.request(req)
        //        reqbuilder.body(hyper::Body::from(serde_json::ser::to_string(data)))
        //    //.json(data)
        //    .send().await
        //    .map_err(|e| ApiError::Network(request.clone(), e.to_string()))?;
        
        Self::handle_response(response, request).await
    }
}

macro_rules! make_get_endpoint {
    ($name:ident<$type:ty>($endpoint:literal)) => {
        pub async fn $name(&self) -> Result<$type, ApiError> {
            self.basic_get_request(AboutRequest{ 
                endpoint: String::from($endpoint),
                verb: String::from("GET"),
                post_data: None
            }).await
        }
    };
}

macro_rules! make_post_endpoint {
    ($name:ident<$intype:ty,$type:ty>($endpoint:literal)) => {
        pub async fn $name(&self, data: &$intype) -> Result<$type, ApiError> {
            self.basic_post_request(AboutRequest{ 
                endpoint: String::from($endpoint),
                verb: String::from("POST"),
                post_data: Some(format!("{:#?}", data))
            }, data).await
        }
    };
}

//This is the rest of the implementation, which are all the actual functions you want to call!
impl ApiContext {
    make_get_endpoint!{get_about<About>("/status")}
    make_get_endpoint!{get_me<User>("/user/me")}
    make_get_endpoint!{get_userprivate<UserPrivate>("/user/privatedata")}

    make_post_endpoint!{post_login<forms::Login,String>("/user/login")}
    make_post_endpoint!{post_register<forms::Register,User>("/user/register")}
    make_post_endpoint!{post_email_sendregistration<forms::EmailGeneric<'_>,bool>("/user/sendregistrationcode")}
    make_post_endpoint!{post_email_recover<forms::EmailGeneric<'_>,bool>("/user/sendpasswordrecovery")}
    make_post_endpoint!{post_register_confirm<forms::RegisterConfirm,String>("/user/confirmregistration")}
    make_post_endpoint!{post_usersensitive<forms::UserSensitive<'_>,String>("/user/privatedata")} //Returns token now
    make_post_endpoint!{post_request<FullRequest,RequestResult>("/request")}
    make_post_endpoint!{post_userupdate<User,User>("/write/user")}
    make_post_endpoint!{post_content<Content,Content>("/write/content")}

    //Some special wrappers

    //This consumes the error and returns "None", since it could just be that the token is stupid. In the future,
    //we may want to alert the user that their token is invalid somewhere, which would require propogating the
    //error result AND checking the status to determine if we need JSON or not...
    pub async fn get_me_safe(&self) -> Option<User>
    {
        //Only run if there IS a token
        match self.user_token {
            //Once we have the token, try it against the api. If there's an error, just print it and move on
            //with apparently "no" user
            Some(_) => match self.get_me().await
            {
                Ok(result) => Some(result),
                Err(_error) => None //Probably need to log at some point!
            }
            None => None
        }
    }

    pub async fn get_user_private_safe(&self) -> Option<UserPrivate>
    {
        //Only run if there IS a token
        match self.user_token {
            //Once we have the token, try it against the api. If there's an error, just print it and move on
            //with apparently "no" user
            Some(_) => match self.get_userprivate().await
            {
                Ok(result) => Some(result),
                Err(_error) => None //Probably need to log at some point!
            }
            None => None
        }
    }
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