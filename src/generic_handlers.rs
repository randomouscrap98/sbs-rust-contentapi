use std::convert::Infallible;

use pages::LinkConfig;
use warp::reject::InvalidQuery;
use warp::{Rejection, Reply};
use warp::body::BodyDeserializeError;
use warp::hyper::{StatusCode};

use crate::{errors::*, SESSIONCOOKIE};

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let code: StatusCode;
    let message: String;
    if let Some(error) = err.find::<ErrorWrapper>() {
        match &error.error {
            pages::Error::Api(apierr) => { 
                code = StatusCode::from_u16(apierr.to_status()).unwrap();
                message = apierr.to_verbose_string();
            }
            pages::Error::Other(otherr) => {
                code = StatusCode::INTERNAL_SERVER_ERROR;
                message = otherr.clone();
            }
        }
    }
    else if let Some(error) = err.find::<BodyDeserializeError>() {
        code = StatusCode::BAD_REQUEST;
        message = error.to_string();    
    }
    else if let Some(error) = err.find::<InvalidQuery>() {
        code = StatusCode::BAD_REQUEST;
        message = error.to_string();
    }
    else {
        code = StatusCode::NOT_FOUND;
        message = String::from("Couldn't figure out what to do with this URL!");
        println!("UNHANDLED REJECTION (404): {:?}", err);
    }
    println!("Rejecting as {}: {}", code, message);
    Ok(warp::reply::with_status(message, code))
}


pub fn handle_response(response: pages::Response, link_config: &LinkConfig) -> Result<impl Reply, Rejection>
{
    handle_response_with_token(response, link_config, None, 0)
}

pub fn handle_response_with_token(response: pages::Response, link_config: &LinkConfig, token: Option<String>, expire: i64) -> Result<impl Reply, Rejection>
{
    //Have to begin the builder here? Then if there's a token, add the header?
    let mut builder = warp::http::Response::builder();

    if let Some(token) = token {
        builder = builder.header("set-cookie", format!("{}={}; Max-Age={}; Path=/; SameSite=Strict", SESSIONCOOKIE, token, expire));
    }

    match response {
        pages::Response::Redirect(url) => {
            builder = builder.status(303).header("Location", format!("{}{}", link_config.http_root, url));
            Ok(errwrap!(builder.body(String::from("")))?) 
        },
        pages::Response::Render(page) => {
            builder = builder.status(200).header("Content-Type", "text/html");
            Ok(errwrap!(builder.body(page))?)
        }
    }
}