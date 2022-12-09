use std::{net::SocketAddr, convert::Infallible, sync::Arc};

use contentapi::endpoints::ApiContext;
use contentapi::endpoints::ApiError;
use pages::LinkConfig;
use warp::hyper::StatusCode;
use warp::{Filter, path::FullPath, reject::Reject, Rejection, Reply};

use pages::{UserConfig, MainLayoutData};

mod config;
mod conversion;

static CONFIGNAME : &str = "settings";
static SESSIONCOOKIE: &str = "sbs-rust-contentapi-session";

#[derive(Debug)]
pub enum MyError {
    Api(ApiError)
}

impl Reject for MyError {}

//This is so stupid. Oh well
macro_rules! apierr {
    ($apierr:expr) => {
        $apierr.map_err(|e| MyError::Api(e))
    };
}


//impl From<ApiError> for MyError {
//    fn from(other: ApiError) -> Self {
//        MyError::Api(other)
//    }
//}

//impl From<MyError> for Rejection {
//    fn from(other: MyError) -> Self {
//        //This is so stupid, honestly. All this wrapping? What's the point??
//        warp::reject::custom(other)
//    }
//}

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


//oof
#[derive(Clone)]
struct GlobalState {
    link_config: LinkConfig,
    cache_bust: String,
    config: Config
}

impl GlobalState {
    async fn context_map<'a>(&'a self, path: FullPath) -> Result<(MainLayoutData,ApiContext), Rejection> {
        let context = ApiContext::new(self.config.api_endpoint.clone(), None);
        let layout_data = MainLayoutData {
            config: self.link_config.clone(),
            user_config: UserConfig::default(),
            current_path: String::from(path.as_str()),
            user: None,
            about_api: apierr!(context.get_about().await)?,
            cache_bust: self.cache_bust.clone()
        };
        Ok((layout_data, ApiContext::new(self.config.api_endpoint.clone(), None)))
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
    let favicon_route = warp::path("favicon.ico").and(warp::fs::file("static/resources/favicon.ico"));
    let index_state = global_state.clone();

    let index_route = warp::get()
        .and(warp::path::end())
        .and(warp::path::full())
        .and_then(move |path| {
            let whatever = index_state.clone();
            async move { whatever.context_map(path).await }
        })
        .map(|(data,_context)| warp::reply::html(pages::index::index(data)));

    warp::serve(static_route.or(favicon_route)
        .or(index_route)
        .recover(handle_rejection)
    ).run(global_state.config.host_address.parse::<SocketAddr>().unwrap()).await;
}

async fn handle_rejection(_err: Rejection) -> Result<impl Reply, Infallible> {
    Ok(warp::reply::with_status("Well, that failed", StatusCode::BAD_REQUEST))
}