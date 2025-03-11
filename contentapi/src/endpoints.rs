use core::fmt::Debug;

use serde::de::DeserializeOwned;
use serde::Serialize;

use super::*;

//There is some "context" that represents a current user and their client connection,
//as well as the api endpoint to connect to. This is used to craft requests on your behalf

//These are the specific types of errors we'll care about from the api. In all instances, the String
//is a minimal amount of data to show the users. The rest is for logging
//#[derive(Error, Debug)]
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

impl From<Box<dyn std::error::Error>> for ApiError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        Self::Other(error.to_string())
    }
}

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

    //Construct a basic GET request to the given endpoint (including ?params) using the given
    //request context. Automatically add bearer headers and all that. Errors on the appropriate
    //status codes, message is assumed to be parsed from body
    pub async fn basic_get_request<T: DeserializeOwned>(
        &self,
        request: AboutRequest,
    ) -> Result<T, ApiError> {
        let reqbuilder = self.get_request_builder(&request, hyper::Method::GET)?;
        let req = noreqerr!(reqbuilder.body(hyper::Body::empty()), request)?;

        //Mapping the request error to a string is PERFECTLY ok in this library because these errors are
        //NOT from stuff like 400 or 500 statuses, they're JUST from network errors (it's localhost so
        //it should never happen, and I'm fine with funky output for the few times there are downtimes)
        let response = neterr!(self.client.request(req).await, request)?;

        Self::handle_response(response, request).await
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

        #[cfg(feature = "postdump")]
        println!("Request: {:?}", &req);

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
