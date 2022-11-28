
use std::net::IpAddr;

use rocket::outcome::{try_outcome, IntoOutcome, Outcome};
use reqwest::Client;
use rocket::{http::CookieJar, request::FromRequest};
use crate::config::Config;

//This is the request context, which rocket may have systems for but I don't want to deal with that
pub struct Context
{
    pub config: Config,
    pub client: Client,
    pub user_token: Option<String>,
    pub client_ip: Option<IpAddr>
}

impl Context
{
    pub fn new<'a>(config: &Config, jar: &CookieJar<'_>, ip: Option<IpAddr>) -> Self
    {
        Self
        {
            client: reqwest::Client::new(),
            config: config.clone(),
            user_token: jar.get(&config.token_cookie_key).and_then(|cookie| Some(String::from(cookie.value()))),
            client_ip: ip
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Context {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r rocket::Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        //Assuming not expensive
        //let config = try_outcome!(request.guard::<rocket::State<Config>>().await);
        //let config = //request.rocket().state::<Config>().or_forward(());//request.guard::<&rocket::State<Config>>().await;
        let jar = request.cookies(); //try_outcome!(request.guard::<&CookieJar<'_>>().await);
        let ip = request.client_ip(); //try_outcome!(request.guard::<&CookieJar<'_>>().await);
        
        //I honestly don't know how to do this, I'm going crazy
        if let Some(config) = request.rocket().state::<Config>() {
            Outcome::Success(Context::new(config, jar, ip))
        }
        else {
            //IDK, the example had it
            Outcome::Forward(())//Failure("Couldn't pull config?")
        }
        //.map(|config| Context::new(config, jar, ip)).or_forward(())

        //let context = request.local_cache_async(async {
        //    let jar = request.guard::<&rocket::State<Config>>().await.succeeded()?;
        //    request.cookies()
        //        .get_private("user_id")
        //        .and_then(|cookie| cookie.value().parse().ok())
        //        .and_then(|id| db.get_user(id).ok())
        //}).await;

        //context.as_ref().or_forward(())
        //rocket::outcome::Outcome::Success(Context::new(config, jar, ip))
    }
}