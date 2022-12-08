use std::net::IpAddr;

use rocket::outcome::Outcome;
use reqwest::Client;
use rocket::request::FromRequest;
use crate::{config::Config, api_data::User, api};

//This is the request context, which rocket may have systems for but I don't want to deal with that.
//Also, notice that all the data here is specifically owned by Context and isn't borrowed (no references);
//this is so I can count on the lifetime of Context and its fields to always be longer than any route
//without having to annotate lifetimes. It's not even the complication, I just... don't want to annotate
//the lifetimes for all 30 or whatever routes that use Context (and all the function calls). SO MANY things
//depend on this one little class that annotating the lifetime would actually cause significant annotation bloat
pub struct Context
{
    pub config: Config,
    pub client: Client,
    pub user_token: Option<String>,
    pub client_ip: Option<IpAddr>,
    pub route_path: String,
    pub route_uri: String,
    pub current_user: Option<User>,
    pub init: InitData
}

//Data about the runtime of this website (on initialization, should only be generated once)
#[derive(Clone)]
pub struct InitData {
    pub boot_time: chrono::DateTime<chrono::Utc>
}


#[rocket::async_trait]
impl<'r> FromRequest<'r> for Context {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r rocket::Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        //Assuming not expensive
        let jar = request.cookies(); 
        let client_ip = request.client_ip(); 
        let uri = request.uri();
        let path = uri.path();

        //I honestly don't know how to do this, I'm going crazy
        if let Some(config) = request.rocket().state::<Config>() {
            if let Some(init_data) = request.rocket().state::<InitData>() {
                let mut context = Context {
                    config: config.clone(), //These clones aren't necessary
                    init: init_data.clone(),
                    client: reqwest::Client::new(),
                    user_token: jar.get(&config.token_cookie_key).and_then(|cookie| Some(String::from(cookie.value()))),
                    route_path: path.to_string(),
                    route_uri: uri.to_string(),
                    current_user: None,
                    client_ip,
                };
                context.current_user = api::get_user_safe(&context).await;
                return Outcome::Success(context);
            }
        }

        //IDK, the example had it
        Outcome::Forward(())
    }
}