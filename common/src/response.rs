
use contentapi::endpoints;

// -------------------------------------
// *     Response/Error from pages     *
// -------------------------------------

#[derive(Debug)]
pub enum Response {
    Render(String), //string is the markup
    RenderWithStatus(String, u16),  //string is the markup, status is the status code returned
    MessageWithStatus(String, u16), //Not an html page, just a message
    Redirect(String)
}

#[derive(Debug)]
pub enum Error {
    Api(contentapi::endpoints::ApiError),
    Data(String, String), //First string is error to output, second is the data itself (don't print for user)
    NotFound(String),   //Normal "not found" error
    User(String),       //A user-generated error, usually related to request. Should produce 400
    Other(String) //Something "general" happened, who the heck knows?
}

impl From<endpoints::ApiError> for Error {
    fn from(error: endpoints::ApiError) -> Self {
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
    pub fn to_user_string(&self) -> String {
        match self {
            Self::Api(error) => error.to_user_string(),
            Self::Other(error) => error.clone(),
            Self::User(error) => error.clone(),
            Self::NotFound(error) => error.clone(),
            Self::Data(error, _data) => error.clone()
        }
    }
}


/// Response is powerful enough to represent both errors and responses, so this function flattens
/// a result of either response or error into just a response. Why have both? I don't know... sometimes something
/// is an error and you want to know!!
pub fn flatten(result: Result<Response, Error>) -> Response
{
    match result
    {
        Ok(response) => response,
        Err(error) => {
            match error
            {
                Error::Api(apierr) => Response::MessageWithStatus(apierr.to_verbose_string(), apierr.to_status()),
                Error::Other(otherr) => Response::MessageWithStatus(otherr.clone(), 500),
                Error::NotFound(otherr) => Response::MessageWithStatus(otherr.clone(), 404),
                Error::User(otherr) => Response::MessageWithStatus(otherr.clone(), 400),
                Error::Data(derr,data) => {
                    println!("DATA ERROR: {}\n{}", derr, data);
                    Response::MessageWithStatus(derr.clone(), 500)
                }
            }
        }
    }
}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for Response {
    fn into_response(self) -> axum::response::Response {
        match self {
            Response::Render(html) => axum::response::Html(html).into_response(),
            Response::RenderWithStatus(html, status) => 
                (
                    axum::http::StatusCode::from_u16(status).unwrap(),
                    [(axum::http::header::CONTENT_TYPE, "text/html")],
                    html,
                ).into_response(),
            Response::MessageWithStatus(msg, status) => 
                (
                    axum::http::StatusCode::from_u16(status).unwrap(),
                    msg,
                ).into_response(),
            Response::Redirect(uri) => axum::response::Redirect::to(&uri).into_response()
        }
    }
}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        flatten(Err(self)).into_response()
    }
}