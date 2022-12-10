use std::{net::SocketAddr, convert::Infallible, sync::Arc};

use contentapi::endpoints::{ApiContext, ApiError};
use pages::{LinkConfig, UserConfig, MainLayoutData};

//use warp::hyper::StatusCode;
//use warp::path::FullPath;
//use warp::reject::Reject;
//use warp::{Filter, Rejection, Reply};

mod config;

static CONFIGNAME : &str = "settings";
static SESSIONCOOKIE: &str = "sbs-rust-contentapi-session";

//#[derive(Debug)]
//pub struct ErrorWrapper {
//    error: pages::Error
//}
//
////Just a bunch of stupid repetitive stuff because IMO bad design (can't impl Reject on types that aren't defined in the crate)
//impl Reject for ErrorWrapper {}
//impl From<ApiError> for ErrorWrapper { fn from(error: ApiError) -> Self { Self { error: pages::Error::Api(error) } } }
//impl From<pages::Error> for ErrorWrapper { fn from(error: pages::Error) -> Self { Self { error } } }

//This is so stupid. Oh well
//macro_rules! apierr {
//    ($apierr:expr) => {
//        $apierr.map_err(|e| Into::<ErrorWrapper>::into(e))
//    };
//}

//Why do we need this when we have from? Well, because warp is stupid, that's why! Warp REQUIRES
//that you return a "Rejection" in your Result, but only THEY have the impl for Into<Rejection>, so
//not only do I have to write a wrapper
//fn apierr(error: ApiError) -> ErrorWrapper {
//
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


/// The unchanging configuration for the current runtime. Mostly values read from 
/// config, but some other constructed data too
#[derive(Clone)]
struct GlobalState {
    link_config: LinkConfig,
    cache_bust: String,
    config: Config
}

/// A context generated for each request. Even if the request doesn't need all the data,
/// this context is generated. The global_state is pretty cheap, and nearly all pages 
/// require the api_about in MainLayoutData, which requires the api_context.
struct RequestContext {
    global_state: Arc<GlobalState>,
    api_context: ApiContext,
    layout_data: MainLayoutData
}

//impl RequestContext {
//    async fn generate(state: Arc<GlobalState>, path: FullPath, token: Option<String>) -> Result<Self, ErrorWrapper> {
//        let context = ApiContext::new(state.config.api_endpoint.clone(), token);
//        let layout_data = MainLayoutData {
//            config: state.link_config.clone(),
//            user_config: UserConfig::default(),
//            current_path: String::from(path.as_str()),
//            user: context.get_me_safe().await,
//            about_api: context.get_about().await?,
//            cache_bust: state.cache_bust.clone()
//        };
//        Ok(RequestContext {
//            global_state: state,
//            api_context: context,
//            layout_data
//        })
//    }
//}

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

    //let static_route = warp::path("static").and(warp::fs::dir("static"));
    //let favicon_route = warp::path("favicon.ico").and(warp::fs::file("static/resources/favicon.ico"));

    ////This "state filter" should be placed at the end of your path but before you start collecting your
    ////route-specific data. It will collect the path and the session cookie (if there is one) and create
    ////a context with lots of useful data to pass to all the templates (but not ALL of it like before)
    //let global_for_state = global_state.clone();
    //let state_filter = warp::path::full()
    //    .and(warp::cookie::optional::<String>(SESSIONCOOKIE))
    //    .and_then(move |path, token| {  //Create a closure that takes ownership of map_state to let it infinitely clone
    //        let this_state = global_for_state.clone();
    //        async move { RequestContext::generate(this_state, path, token).await.map_err(|e| Into::<Rejection>::into(e)) }
    //    });
    
    //let global_for_form = global_state.clone();
    //let form_filter = warp::body::content_length_limit(global_for_form.config.body_maxsize as u64);

    ////Lets anybody get the global state (maybe you want some extra config value?)
    ////let global_for_arb = global_state.clone();
    ////let get_global_state = warp::path::any().map(move || global_for_arb.clone());

    //let index_route = warp::get()
    //    .and(warp::path::end())
    //    .and(state_filter.clone())
    //    .map(|context:RequestContext| warp::reply::html(pages::index::render(context.layout_data)));

    //let about_route = warp::get()
    //    .and(warp::path!("about"))
    //    .and(state_filter.clone())
    //    .map(|context:RequestContext| warp::reply::html(pages::about::render(context.layout_data)));

    //let login_route = warp::get()
    //    .and(warp::path!("login"))
    //    .and(state_filter.clone())
    //    .map(|context:RequestContext| warp::reply::html(pages::login::render(context.layout_data, None, None, None)));

    //let login_post_route = warp::post()
    //    .and(warp::path!("login"))
    //    .and(state_filter.clone())
    //    .and(form_filter.clone())
    //    .and(warp::body::form::<pages::login::Login>())
    //    .map(|context:RequestContext,form| warp::reply::html(pages::login::render(context.layout_data, None, None, None)));

    //warp::serve(static_route.or(favicon_route)
    //    .or(index_route)
    //    .or(about_route)
    //    .or(login_route)
    //    .or(login_post_route)
    //    .recover(handle_rejection)
    //).run(global_state.config.host_address.parse::<SocketAddr>().unwrap()).await;
}

//async fn handle_rejection(_err: Rejection) -> Result<impl Reply, Infallible> {
//    Ok(warp::reply::with_status("Well, that failed", StatusCode::BAD_REQUEST))
//}