use std::net::SocketAddr;

use warp::Filter;

mod bbcode;
mod api;
mod config;
mod templates;
mod conversion;
mod routing;
//mod api_data;

//use crate::config::create_config;
//use crate::templates;

static CONFIGNAME : &str = "settings";

//The standard config we want here in this application. This macro is ugly but 
//it produces a config object that can load from a chain of json files
config::create_config!{
    Config, OptConfig => {
        api_endpoint: String,
        http_root: String,
        api_fileraw : String,
        token_cookie_key: String,
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

#[tokio::main]
async fn main() {

    //Our env is passed on the command line. If none is, we pass "None" so only the base config is read
    let args: Vec<String> = std::env::args().collect();
    let environment = args.get(1).map(|x| &**x); //The compiler told me to do this

    let config = Config::read_with_environment_toml(CONFIGNAME, environment);

    println!("{:#?}", config);

    // GET /hello/warp => 200 OK with body "Hello, warp!"
    //let hello = warp::path!("hello" / String)
    //    .map(|name| format!("Hello, {}!", name));

    //println!("{}", templates::index::html!().into_string());
    
    warp::serve(routing::get_all_routes())
        .run(config.host_address.parse::<SocketAddr>().unwrap())
        .await;
}