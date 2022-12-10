use std::{net::SocketAddr, convert::Infallible, sync::Arc};

use axum::{extract::Path, response::IntoResponse, body, http::StatusCode, routing::get};
use include_dir::{include_dir, Dir};
use contentapi::endpoints::{ApiContext, ApiError};
//use errors::ErrorWrapper;
use pages::{LinkConfig, UserConfig, MainLayoutData};

//use warp::hyper::{StatusCode};
//use warp::path::FullPath;
//use warp::{Filter, Rejection, Reply};

//use crate::errors::{ApiErrorWrapper, apierrwrap};

//use crate::errors::errwrap;

mod config;
//mod errors;


//LMAO embed all the static files! That's kinda cool ngl
static STATIC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/static");


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

impl RequestContext {
    async fn generate(state: Arc<GlobalState>, path: String, token: Option<String>) -> Result<Self, ApiError> {
        let context = ApiContext::new(state.config.api_endpoint.clone(), token);
        let layout_data = MainLayoutData {
            config: state.link_config.clone(),
            user_config: UserConfig::default(),
            current_path: path, //String::from(path.as_str()),
            user: context.get_me_safe().await,
            about_api: context.get_about().await?,
            cache_bust: state.cache_bust.clone()
        };
        Ok(RequestContext {
            global_state: state,
            api_context: context,
            layout_data
        })
    }
}

//enum ContextOrElse {
//    Context(RequestContext),
//    Error(ApiError)
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

    let address = global_state.config.host_address.parse::<SocketAddr>().unwrap();

    let app = axum::Router::new()
        .route("/favicon.ico", get(favicon_path))
        .route("/static/*path", get(static_path))
        .route("/index", get(index_path))
        .with_state(global_state);

    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// Taken almost verbatim from https://bloerg.net/posts/serve-static-content-with-axum/
async fn static_path_internal(path: &str) -> impl IntoResponse {
    println!("Path: {}", path);
    let mime_type = mime_guess::from_path(path).first_or_text_plain();

    if let Some(file) = STATIC_DIR.get_file(path) {
        axum::response::Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", mime_type.as_ref())
            .body(body::boxed(body::Full::from(file.contents())))
            .unwrap() //We leave this as "unwrap" because it will only fail with bad responses, but we constructed the response ourselves?
    }
    else {
        axum::response::Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(body::boxed(body::Empty::new()))
            .unwrap()
    }
}

async fn static_path(Path(path): Path<String>) -> impl IntoResponse {
    static_path_internal(path.trim_start_matches("/")).await
}

async fn favicon_path() -> impl IntoResponse {
    static_path_internal("resources/favicon.ico").await
}

//async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
//    let code;
//    let message;
//    if err.is_not_found() {
//        code = StatusCode::NOT_FOUND;
//        message = "Couldn't figure out what to do with this URL!";
//    } else if let Some(ErrorWrapper) = err.find() {
//        match error {
//
//        }
//        code = StatusCode::BAD_REQUEST;
//        message = "DIVIDE_BY_ZERO";
//    } else {
//        code = StatusCode::INTERNAL_SERVER_ERROR;
//        message = 
//    }
//    Ok(warp::reply::with_status("Well, that failed", StatusCode::BAD_REQUEST))
//}

fn handle_response(response: pages::Response, link_config: &LinkConfig, token: Option<String>, expire: i64) -> impl IntoResponse {
{
    //Have to begin the builder here? Then if there's a token, add the header?
    let mut builder = axum::response::Response::builder();

    if let Some(token) = token {
        builder = builder.header("set-cookie", format!("{}={}; Max-Age={}; SameSite=Strict", SESSIONCOOKIE, token, expire));
    }

    match response {
        pages::Response::Redirect(url) => {
            builder = builder.status(303).header("Location", format!("{}{}", link_config.http_root, url));
            Ok(errwrap!(builder.body(String::from("")))?) 
        },
        pages::Response::Render(page) => {
            builder = builder.status(200).header("Content-Type", "text/html");
            Ok(errwrap!(builder.body(page))?)
        }
    }
}
