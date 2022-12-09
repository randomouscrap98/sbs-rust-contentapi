use std::{net::SocketAddr, convert::Infallible, sync::Arc};

use contentapi::endpoints::{ApiContext, ApiError};
use pages::{LinkConfig, UserConfig, MainLayoutData};

use warp::hyper::StatusCode;
use warp::path::FullPath;
use warp::reject::Reject;
use warp::{Filter, Rejection, Reply};

mod sbsforms;
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
    async fn context_map<'a>(&'a self, path: FullPath, token: Option<String>) -> Result<(MainLayoutData,ApiContext), Rejection> {
        let context = ApiContext::new(self.config.api_endpoint.clone(), token);
        let layout_data = MainLayoutData {
            config: self.link_config.clone(),
            user_config: UserConfig::default(),
            current_path: String::from(path.as_str()),
            user: context.get_me_safe().await,
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

    //Set up the SINGULAR global state, which will be passed around with a counting reference.
    //So when you see "clone" on this, it's not actually cloning all the data, it's just making
    //a new pointer and incrementing a count.
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

    let static_route = warp::path("static").and(warp::fs::dir("static"));
    let favicon_route = warp::path("favicon.ico").and(warp::fs::file("static/resources/favicon.ico"));

    //This "state filter" should be placed at the end of your path but before you start collecting your
    //route-specific data. It will collect the path and the session cookie (if there is one) and create
    //a context with lots of useful data to pass to all the templates (but not ALL of it like before)
    let map_state = global_state.clone();
    let state_filter = warp::path::full()
        .and(warp::cookie::optional::<String>(SESSIONCOOKIE))
        .and_then(move |path, token| {  //Create a closure that takes ownership of map_state to let it infinitely clone
            let this_state = map_state.clone();
            async move { this_state.context_map(path, token).await }
        });
    
    let form_state = global_state.clone();
    let form_filter = warp::body::content_length_limit(form_state.config.body_maxsize as u64);
        //.and(warp::body::form());

    let index_route = warp::get()
        .and(warp::path::end())
        .and(state_filter.clone())
        .map(|(data,_context)| warp::reply::html(pages::index::render(data)));

    let about_route = warp::get()
        .and(warp::path!("about"))
        .and(state_filter.clone())
        .map(|(data,_context)| warp::reply::html(pages::about::render(data)));

    let login_route = warp::get()
        .and(warp::path!("login"))
        .and(state_filter.clone())
        .map(|(data,_context)| warp::reply::html(pages::login::render(data, None, None, None)));

    let login_post_route = warp::post()
        .and(warp::path!("login"))
        .and(state_filter.clone())
        .and(form_filter.clone())
        .and(warp::body::form::<sbsforms::Login>())
        .map(|(data,_context),form| warp::reply::html(pages::login::render(data, None, None, None)));

    warp::serve(static_route.or(favicon_route)
        .or(index_route)
        .or(about_route)
        .or(login_route)
        .or(login_post_route)
        .recover(handle_rejection)
    ).run(global_state.config.host_address.parse::<SocketAddr>().unwrap()).await;
}

async fn handle_rejection(_err: Rejection) -> Result<impl Reply, Infallible> {
    Ok(warp::reply::with_status("Well, that failed", StatusCode::BAD_REQUEST))
}