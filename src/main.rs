use std::net::SocketAddr;

use pages::LinkConfig;
use reqwest::Client;
use warp::{Filter, path::FullPath};

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

struct Context<'a> {
    api_url: &'a str,
    client: Client,
}

impl api::endpoints::Context for Context<'_> {
    fn get_api_url(&self) -> &str {
        self.api_url
    }
    fn get_client(&self) -> &Client {
        &self.client
    }
    fn get_user_token(&self) -> Option<&str> {
        None
    }
}

#[tokio::main]
async fn main() {

    //Our env is passed on the command line. If none is, we pass "None" so only the base config is read
    let args: Vec<String> = std::env::args().collect();
    let environment = args.get(1).map(|x| &**x); //The compiler told me to do this

    let config = Config::read_with_environment_toml(CONFIGNAME, environment);

    println!("{:#?}", config);

    let cache_bust = chrono::offset::Utc::now().to_string();

    //Basically "global" state
    let link_config = {
        let root = config.http_root.clone();
        LinkConfig {
            static_root: format!("{}/static", &root),
            resource_root: format!("{}/static/resources", &root),
            file_root: config.api_fileraw.clone(),
            http_root: root
        }
    };

    let static_route = warp::path("static").and(warp::fs::dir("static"));

    let context_map = |path: FullPath| async {
        let context = Context {
            api_url: &config.api_endpoint,
            client: reqwest::Client::new(),
        };
        let layout_data = MainLayoutData {
            config: &link_config,
            user_config: UserConfig::default(),
            current_path: String::from(path.as_str()),
            user: None,
            about_api: &api::endpoints::get_about(&context).await?,
            cache_bust: &cache_bust
        };
        Ok((layout_data, context))
    };

    let index_route = warp::get()
        .and(warp::path::end())
        .and(warp::path::full())
        .and_then(context_map)
        .map(|(data,context)| pages::index::index(data));

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
    ).run(config.host_address.parse::<SocketAddr>().unwrap()).await;
}
