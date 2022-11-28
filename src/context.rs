
use std::net::{IpAddr, SocketAddr};

use reqwest::Client;
use rocket::http::CookieJar;
use crate::config::Config;

//This is the request context, which rocket may have systems for but I don't want to deal with that
pub struct Context
{
    pub config: Config,
    pub client: Client,
    pub user_token: Option<String>,
    pub client_ip: IpAddr 
}

impl Context
{
    pub fn new<'a>(config: &Config, jar: &CookieJar<'_>, ip: IpAddr) -> Self
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