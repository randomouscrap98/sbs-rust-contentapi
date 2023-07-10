use std::{net::SocketAddr, sync::Arc};

use bbscope::{BBCode, BBCodeTagConfig, BBCodeLinkTarget};
use chrono::SecondsFormat;
use common::LinkConfig;

mod state;
mod routing;

use crate::state::*;

static CONFIGNAME : &str = "settings";

//The standard config we want here in this application. This macro is ugly but 
//it produces a config object that can load from a chain of json files
onestop::create_config!{
    Config, OptConfig => {
        api_endpoint: String,
        http_root: String,
        api_fileraw : String,
        default_cookie_expire: i32,
        long_cookie_expire: i32,
        default_imagebrowser_count: i32,
        default_category_threads : i32,
        default_display_threads : i32,
        default_display_posts : i32,
        default_display_pages : i32,
        default_activity_count: i32,
        forum_category_order: Vec<String>,
        //file_maxsize: i32,
        body_maxsize: i32, //this can be used for a lot of things, I don't really care
        host_address: String,
    }
}

#[tokio::main]
async fn main() 
{
    let config = {
        //Our env is passed on the command line. If none is, we pass "None" so only the base config is read
        let args: Vec<String> = std::env::args().collect();
        let environment = args.get(1).map(|x| &**x); //The compiler told me to do this

        let config = Config::read_with_environment_toml(CONFIGNAME, environment);
        println!("Environment: {}\n{:#?}", environment.unwrap_or(""), config);
        config
    };

    let bbcode = {
        let mut config = BBCodeTagConfig::extended();
        config.link_target = BBCodeLinkTarget::None;
        config.newline_to_br = false;
        BBCode::from_config(config, None).unwrap()
    };

    //Set up the SINGULAR global state, which will be passed around with a counting reference.
    //So when you see "clone" on this, it's not actually cloning all the data, it's just making
    //a new pointer and incrementing a count.
    let global_state = Arc::new(GlobalState {
        bbcode,
        link_config : {
            let root = config.http_root.clone();
            LinkConfig {
                static_root: format!("{}/static", &root),
                resource_root: format!("{}/static/resources", &root),
                file_root: format!("{}/raw", config.api_fileraw),
                file_upload_root: format!("{}/low", config.api_fileraw),
                http_root: root,
                cache_bust : chrono::offset::Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true) //.to_string()
            }
        },
        config
    });

    let address = global_state.config.host_address.parse::<SocketAddr>().unwrap();
    let app = routing::get_all_routes(global_state.clone());

    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap();

}

