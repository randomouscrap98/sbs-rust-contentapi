use std::{net::SocketAddr, sync::Arc};

use chrono::SecondsFormat;
use contentapi::endpoints::{ApiContext, ApiError};
use pages::{LinkConfig, UserConfig, MainLayoutData};

use serde::Deserialize;
use warp::path::FullPath;
use warp::{Filter, Rejection};

mod config;
mod errors;
mod generic_handlers;

use crate::errors::{ErrorWrapper, errwrap};
use crate::generic_handlers::*;

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
    async fn generate(state: Arc<GlobalState>, path: FullPath, token: Option<String>) -> Result<Self, ApiError> {
        let context = ApiContext::new(state.config.api_endpoint.clone(), token);
        let layout_data = MainLayoutData {
            config: state.link_config.clone(),
            user_config: UserConfig::default(),
            current_path: String::from(path.as_str()),
            user: context.get_me_safe().await,
            about_api: context.get_about().await?
        };
        Ok(RequestContext {
            global_state: state,
            api_context: context,
            layout_data
        })
    }
}

//Silly thing to limit a route by a single flag present (must be i8)
macro_rules! qflag {
    ($flag:ident) => {
        {
            #[allow(dead_code)]
            #[derive(Deserialize)]
            struct QueryFlag { $flag: i8 }

            warp::query::<QueryFlag>()
        } 
    };
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
        link_config : {
            let root = config.http_root.clone();
            LinkConfig {
                static_root: format!("{}/static", &root),
                resource_root: format!("{}/static/resources", &root),
                file_root: config.api_fileraw.clone(),
                http_root: root,
                cache_bust : chrono::offset::Utc::now().to_string()
            }
        },
        config
    });

    let address = global_state.config.host_address.parse::<SocketAddr>().unwrap();

    let static_route = warp::path("static").and(warp::fs::dir("static"));
    let favicon_route = warp::path("favicon.ico").and(warp::fs::file("static/resources/favicon.ico"));

    //This "state filter" should be placed at the end of your path but before you start collecting your
    //route-specific data. It will collect the path and the session cookie (if there is one) and create
    //a context with lots of useful data to pass to all the templates (but not ALL of it like before)
    let global_for_state = global_state.clone();
    let state_filter = warp::path::full()
        .and(warp::method())
        .and(warp::cookie::optional::<String>(SESSIONCOOKIE))
        .and_then(move |path, method, token| {  //Create a closure that takes ownership of map_state to let it infinitely clone
            println!("[{}] {} - {:?}", chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true), &method, &path);
            let this_state = global_for_state.clone();
            async move { 
                errwrap!(RequestContext::generate(this_state, path, token).await)
            }
        }).boxed();
    
    let global_for_form = global_state.clone();
    let form_filter = warp::body::content_length_limit(global_for_form.config.body_maxsize as u64);

    macro_rules! warp_get {
        ($filter:expr, $map:expr) => {
            warp::get()
                .and($filter)
                .and(state_filter.clone())
                .map($map)
                .boxed()
        };
    }

    macro_rules! warp_get_async {
        ($filter:expr, $map:expr) => {
            warp::get()
                .and($filter)
                .and(state_filter.clone())
                .and_then($map)
                .boxed()
        };
    }

    let index_route = warp_get!(warp::path::end(), 
        |context:RequestContext| warp::reply::html(pages::index::render(context.layout_data)));

    let about_route = warp_get!(warp::path!("about"),
        |context:RequestContext| warp::reply::html(pages::about::render(context.layout_data)));

    let login_route = warp_get!(warp::path!("login"),
        |context:RequestContext| warp::reply::html(pages::login::render(context.layout_data, None, None, None)));

    let logout_route = warp_get_async!(warp::path!("logout"),
        |context:RequestContext| async move {
            //Logout is a Set-Cookie to empty string with Max-Age set to 0, then redirect to root
            handle_response_with_token(pages::Response::Redirect(format!("{}/",&context.global_state.link_config.http_root)),
                &context.global_state.link_config, Some(String::from("")), 0)
        });

    let activity_route = warp_get!(warp::path!("activity"),
        |context:RequestContext| warp::reply::html(pages::activity::render(context.layout_data)));

    let search_route = warp_get!(warp::path!("search"),
        |context:RequestContext| warp::reply::html(pages::search::render(context.layout_data)));

    let recover_route = warp_get!(warp::path!("recover"),
        |context:RequestContext| warp::reply::html(pages::recover::render(context.layout_data, None, None)));

    let userhome_get_route = warp_get_async!(warp::path!("userhome"),
        |context:RequestContext| {
            async move {
                handle_response(
                    errwrap!(pages::userhome::get_render(context.layout_data, &context.api_context).await)?,
                    &context.global_state.link_config
                )
            }
        }); 

    let imagebrowser_route = warp_get_async!(
        warp::path!("widget" / "imagebrowser")
            .and(warp::query::<pages::widget_imagebrowser::Search>()
                .or(warp::any().map(|| pages::widget_imagebrowser::Search::default()))
                .unify()),
        |search:pages::widget_imagebrowser::Search, context:RequestContext| {
            async move {
                handle_response(
                    errwrap!(pages::widget_imagebrowser::query_render(context.layout_data, 
                        &context.api_context, search, context.global_state.config.default_imagebrowser_count).await)?,
                        &context.global_state.link_config)
            }
        });

    let login_post = warp::any()
        .and(warp::body::form::<pages::login::Login>())
        .and(state_filter.clone())
        .and_then(|form: pages::login::Login, context: RequestContext| {
            let login = form.to_api_login(
                context.global_state.config.default_cookie_expire, 
                context.global_state.config.long_cookie_expire);
            async move {
                let (response,token) = pages::login::post_login_render(context.layout_data, &context.api_context, &login).await;
                handle_response_with_token(response, &context.global_state.link_config, token, login.expireSeconds)
            }
        }).boxed();
    
    let recover_post = warp::any()
        .and(qflag!(recover)) 
        .and(warp::body::form::<pages::EmailGeneric>())
        .and(state_filter.clone())
        .and_then(|_query, form: pages::EmailGeneric, context: RequestContext| {
            async move {
                let response = pages::login::post_login_recover(context.layout_data, &context.api_context, &form).await;
                handle_response(response, &context.global_state.link_config)
            }
        }).boxed();

    //ALL post routes!
    let login_post_route = warp::post()
        .and(warp::path!("login"))
        .and(form_filter.clone())
        .and(recover_post.or(login_post));

    warp::serve(
        static_route.boxed()
        .or(favicon_route)
        .or(index_route)
        .or(activity_route)
        .or(search_route)
        .or(about_route)
        .or(login_route)
        .or(login_post_route)
        .or(logout_route)
        .or(recover_route)
        .or(imagebrowser_route)
        .or(userhome_get_route)
        .recover(handle_rejection)
    ).run(address).await;
}

