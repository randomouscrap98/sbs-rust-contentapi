
use std::net::IpAddr;

use rocket::outcome::Outcome;
use reqwest::Client;
use rocket::request::FromRequest;
use crate::config::Config;

//This is the request context, which rocket may have systems for but I don't want to deal with that
pub struct Context
{
    pub config: Config,
    pub client: Client,
    pub user_token: Option<String>,
    pub client_ip: Option<IpAddr>,
    pub route_path: String
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Context {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r rocket::Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        //Assuming not expensive
        let jar = request.cookies(); 
        let client_ip = request.client_ip(); 
        let path = request.uri().path();

        //I honestly don't know how to do this, I'm going crazy
        if let Some(config) = request.rocket().state::<Config>() {
            Outcome::Success(Context {
                config: config.clone(),
                client: reqwest::Client::new(),
                user_token: jar.get(&config.token_cookie_key).and_then(|cookie| Some(String::from(cookie.value()))),
                route_path: String::from(path.as_str()),
                client_ip,
            })
        }
        else {
            //IDK, the example had it
            Outcome::Forward(())
        }
    }
}