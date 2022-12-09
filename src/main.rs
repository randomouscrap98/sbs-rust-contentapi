use std::{net::SocketAddr, convert::Infallible, sync::Arc};

use api::endpoints::ApiError;
use pages::LinkConfig;
use reqwest::{Client, StatusCode};
use warp::{Filter, path::FullPath, reject::Reject, Rejection, Reply};

use crate::pages::{UserConfig, MainLayoutData};

mod bbcode;
mod api;
mod config;
//mod templates;
mod conversion;
mod routing;
mod pages;
//mod api_data;

//use crate::config::create_config;
//use crate::templates;

static CONFIGNAME : &str = "settings";
static SESSIONCOOKIE: &str = "sbs-rust-contentapi-session";

//The standard config we want here in this application. This macro is ugly but 
//it produces a config object that can load from a chain of json files
config::create_config!{
    Config, OptConfig => {
        api_endpoint: String,
        http_root: String,
        api_fileraw : String,
        //token_cookie_key: String,
        default_cookie_expire: i32,
        long_cookie_expire: i32,
        default_imagebrowser_count: i32,
        default_category_threads : i32,
        default_display_threads : i32,
        default_display_posts : i32,
        forum_category_order: Vec<String>,
        //file_maxsize: i32,
        body_maxsize: i32, //this can be used for a lot of things, I don't really care
        host_address: String,
    }
}

//impl From<&Config> for LinkConfig<'_> {
//    fn from(config: &Config) -> Self {
//        LinkConfig { 
//            http_root: &config.http_root, 
//            static_root: (), 
//            resource_root: (), 
//            file_root: &config.
//        }
//    }
//}

//Warp requires static, so... oh well!
//static config: Config = Config::default();

struct Context {
    api_url: String,
    client: Client,
}

impl api::endpoints::Context for Context {
    fn get_api_url(&self) -> &str {
        &self.api_url
    }
    fn get_client(&self) -> &Client {
        &self.client
    }
    fn get_user_token(&self) -> Option<&str> {
        None
    }
}

//oof
#[derive(Clone)]
struct GlobalState {
    link_config: LinkConfig,
    cache_bust: String,
    config: Config
}

impl GlobalState {
    async fn context_map<'a>(&'a self, path: FullPath) -> Result<(MainLayoutData,Context), Infallible> {

        let context = Context {
            api_url: self.config.api_endpoint.clone(),
            client: reqwest::Client::new(),
        };
        let layout_data = MainLayoutData {
            config: self.link_config.clone(),
            user_config: UserConfig::default(),
            current_path: String::from(path.as_str()),
            user: None,
            about_api: api::endpoints::get_about(&context).await.unwrap(),
            cache_bust: self.cache_bust.clone()
        };
        //Ok((layout_data, context))
        Ok((layout_data, context))
    }
}

#[tokio::main]
async fn main() {

    //Our env is passed on the command line. If none is, we pass "None" so only the base config is read
    let args: Vec<String> = std::env::args().collect();
    let environment = args.get(1).map(|x| &**x); //The compiler told me to do this

    let config = Config::read_with_environment_toml(CONFIGNAME, environment);

    println!("{:#?}", config);

    let global_state = Arc::new(GlobalState {
        cache_bust : chrono::offset::Utc::now().to_string(),
        link_config : {
            let root = config.http_root.clone();
            LinkConfig {
                static_root: format!("{}/static", &root),
                resource_root: format!("{}/static/resources", &root),
                file_root: config.api_fileraw.clone(),
                http_root: root
            }
        },
        config
    });

    //Basically "global" state

    let static_route = warp::path("static").and(warp::fs::dir("static"));
    let ugh = global_state.clone();

    let index_route = warp::get()
        .and(warp::path::end())
        .and(warp::path::full())
        //.and(warp::any().map(|| global_state.clone()))
        .and_then(move |path| {
            let whatever = ugh.clone();
            async move { whatever.context_map(path).await }
        })
        .map(|(data,_context)| pages::index::index(data));

    //let full_website = static_route.or(
    //    warp::cookie::optional(SESSIONCOOKIE).and( //Get the user cookie representing the session, but it's not necessary! At least not here
    //        warp::get()
    //            .and(warp::path::end())
    //            .and(warp::path::full())
    //            .map(|path| {
    //                
    //                pages::index::index()
    //            })
    //    )
    //);

    
    warp::serve(static_route
        .or(index_route)
        .recover(handle_rejection)
    ).run(global_state.config.host_address.parse::<SocketAddr>().unwrap()).await;
}

impl Reject for ApiError {}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    Ok(warp::reply::with_status("Well, that failed", StatusCode::BAD_REQUEST))
}