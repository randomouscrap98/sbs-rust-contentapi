use std::{net::SocketAddr, sync::Arc};
use std::collections::HashMap;

use bbscope::BBCode;
use chrono::SecondsFormat;
use pages::LinkConfig;

use serde::Deserialize;
use warp::filters::BoxedFilter;
use warp::{Filter, Rejection, Reply};

mod config;
mod errors;
mod generic_handlers;
mod state;

use crate::errors::*;
use crate::generic_handlers::*;
use crate::state::*;

static CONFIGNAME : &str = "settings";
static SESSIONCOOKIE: &str = "sbs-rust-contentapi-session";


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
        let mut matchers = BBCode::basics().unwrap(); //this better not fail! It'll fail very early though
        let mut extras = BBCode::extras().unwrap();
        matchers.append(&mut extras);
        BBCode::from_matchers(matchers)
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
                cache_bust : chrono::offset::Utc::now().to_string()
            }
        },
        config
    });

    let address = global_state.config.host_address.parse::<SocketAddr>().unwrap();

    let fs_static_route = warp::path("static").and(warp::fs::dir("static")).boxed();
    let fs_favicon_route = warp::path("favicon.ico").and(warp::fs::file("static/resources/favicon.ico")).boxed();

    //This "state filter" should be placed at the end of your path but before you start collecting your
    //route-specific data. It will collect the path and the session cookie (if there is one) and create
    //a context with lots of useful data to pass to all the templates (but not ALL of it like before)
    let global_for_state = global_state.clone();
    let state_filter = warp::path::full()
        .and(warp::method())
        .and(warp::cookie::optional::<String>(SESSIONCOOKIE))
        .and_then(move |path, method, token| {  //Create a closure that takes ownership of map_state to let it infinitely clone
            println!("[{}] {:>5} - {:?}", chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true), &method, &path);
            let this_state = global_for_state.clone();
            async move { 
                errwrap!(RequestContext::generate(this_state, path, token).await)
            }
        }).boxed();
    
    let global_for_form = global_state.clone();
    let form_filter = warp::body::content_length_limit(global_for_form.config.body_maxsize as u64).boxed();

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

    let get_index_route = warp_get!(warp::path::end(), 
        |context:RequestContext| warp::reply::html(pages::index::render(context.layout_data)));

    let get_about_route = warp_get!(warp::path!("about"),
        |context:RequestContext| warp::reply::html(pages::about::render(context.layout_data)));

    let get_login_route = warp_get!(warp::path!("login"),
        |context:RequestContext| warp::reply::html(pages::login::render(context.layout_data, None, None, None)));

    let get_logout_route = warp_get_async!(warp::path!("logout"),
        |context:RequestContext| async move {
            //Logout is a Set-Cookie to empty string with Max-Age set to 0, then redirect to root
            handle_response_with_token(pages::Response::Redirect(String::from("/")),
                &context.global_state.link_config, Some(String::from("")), 0)
        });

    let get_register_route = warp_get!(warp::path!("register"),
        |context:RequestContext| warp::reply::html(pages::register::render(context.layout_data, None, None, None)));

    let get_registerconfirm_route = warp_get!(warp::path!("register"/"confirm"),
        |context:RequestContext| warp::reply::html(pages::registerconfirm::render(context.layout_data, None, None, None, None, false)));

    let get_recover_route = warp_get!(warp::path!("recover"),
        |context:RequestContext| warp::reply::html(pages::recover::render(context.layout_data, None, None)));

    let get_activity_route = warp_get!(warp::path!("activity"),
        |context:RequestContext| warp::reply::html(pages::activity::render(context.layout_data)));

    let get_bbcodepreview_route = warp_get!(warp::path!("widget" / "bbcodepreview"),
        |context:RequestContext| warp::reply::html(pages::widget_bbcodepreview::render(context.layout_data, &context.global_state.bbcode, None)));

    let post_bbcodepreview_route = warp::post()
        .and(warp::path!("widget" / "bbcodepreview"))
        .and(form_filter.clone())
        .and(warp::body::form::<pages::BasicText>())
        .and(state_filter.clone())
        .map(|form: pages::BasicText, context: RequestContext| {
            warp::reply::html(pages::widget_bbcodepreview::render(context.layout_data, &context.global_state.bbcode, Some(form.text)))
        })
        .boxed();

    let get_search_route = warp_get_async!(
            warp::path!("search")
                .and(warp::query::<HashMap<String, String>>()),
        |search: HashMap<String, String>, context:RequestContext| {
            async move {
                let gc = context.global_state.clone();
                handle_response(
                    errwrap!(pages::search::get_render(
                        context.into(),
                        search
                        ).await)?,
                    &gc.link_config
                )
            }
        });


    #[derive(Deserialize, Debug)]
    struct SimplePage { page: Option<i32> }

    let get_forum_category_route = warp_get_async!(
        warp::path!("forum" / "category" / String)
            .and(warp::query::<SimplePage>()),
        |hash: String, page_struct: SimplePage, context:RequestContext| {
            async move {
                let gc = context.global_state.clone();
                handle_response(
                    errwrap!(pages::forum_category::get_hash_render(
                        context.into(),
                        hash, 
                        gc.config.default_display_threads, 
                        page_struct.page).await)?,
                    &gc.link_config
                )
            }
        }); 

    let get_forum_thread_route = warp_get_async!(
        warp::path!("forum" / "thread" / String)
            .and(warp::query::<SimplePage>()),
        |hash: String, page_struct: SimplePage, context:RequestContext| {
            async move {
                let gc = context.global_state.clone();
                handle_response(
                    errwrap!(pages::forum_thread::get_hash_render(
                        context.into(),
                        hash, 
                        gc.config.default_display_posts, 
                        page_struct.page).await)?,
                    &gc.link_config
                )
            }
        }); 

    let get_forum_post_route = warp_get_async!(
        warp::path!("forum" / "thread" / String / i64),
        |hash: String, post_id: i64, context:RequestContext| {
            async move {
                let gc = context.global_state.clone();
                handle_response(
                    errwrap!(pages::forum_thread::get_hash_postid_render(
                        context.into(),
                        hash, 
                        post_id,
                        gc.config.default_display_posts).await)?,
                    &gc.link_config
                )
            }
        }); 


    let get_user_route = warp_get_async!(warp::path!("user" / String),
        |username: String, context:RequestContext| {
            async move {
                let gc = context.global_state.clone();
                handle_response(
                    errwrap!(pages::user::get_render(context.into(), username).await)?,
                    &gc.link_config
                )
            }
        }); 

    let get_userhome_route = warp_get_async!(warp::path!("userhome"),
        |context:RequestContext| {
            async move {
                let gc = context.global_state.clone();
                handle_response(
                    errwrap!(pages::userhome::get_render(context.into()).await)?,
                    &gc.link_config
                )
            }
        }); 

    let get_imagebrowser_route = warp_get_async!(
        warp::path!("widget" / "imagebrowser")
            .and(warp::query::<pages::widget_imagebrowser::Search>()
                .or(warp::any().map(|| pages::widget_imagebrowser::Search::default()))
                .unify()),
        |search:pages::widget_imagebrowser::Search, context:RequestContext| {
            async move {
                let gc = context.global_state.clone();
                handle_response(
                    errwrap!(pages::widget_imagebrowser::query_render(
                        context.into(),
                        search, 
                        gc.config.default_imagebrowser_count).await)?,
                    &gc.link_config)
            }
        });

    let post_recover_route = warp::post()
        .and(warp::path!("recover"))
        .and(form_filter.clone())
        .and(warp::body::form::<contentapi::forms::UserSensitive>())
        .and(state_filter.clone())
        .and_then(|form: contentapi::forms::UserSensitive, context: RequestContext| {
            async move {
                let gc = context.global_state.clone();
                let (response, token) = pages::recover::post_render(context.into(), &form).await;
                handle_response_with_token(response, &gc.link_config, token, gc.config.default_cookie_expire as i64)
            }
        })
        .boxed();

    let post_register_route = warp::post()
        .and(warp::path!("register"))
        .and(form_filter.clone())
        .and(warp::body::form::<contentapi::forms::Register>())
        .and(state_filter.clone())
        .and_then(|form: contentapi::forms::Register, context: RequestContext| {
            async move {
                let gc = context.global_state.clone();
                let response = pages::register::post_render(context.into(), &form).await;
                handle_response(response, &gc.link_config)
            }
        })
        .boxed();
        
    warp::serve(
            fs_static_route
        .or(fs_favicon_route)
        .or(get_index_route)
        .or(get_activity_route)
        .or(get_search_route)
        .or(get_forum_route(&state_filter)) //HEAVILY multiplexed! Lots of legacy forum paths!
        .or(get_forum_category_route)
        .or(get_forum_thread_route)
        .or(get_forum_post_route)
        .or(get_about_route)
        .or(get_user_route)
        .or(get_userhome_route)
        .or(post_userhome_multi_route(&state_filter, &form_filter)) //Multiplexed! Login OR send recovery!
        .or(get_login_route)
        .or(post_login_multi_route(&state_filter, &form_filter)) //Multiplexed! Login OR send recovery!
        .or(get_logout_route)
        .or(get_register_route)
        .or(post_register_route)
        .or(get_registerconfirm_route)
        .or(post_registerconfirm_multi_route(&state_filter, &form_filter)) //Multiplexed! Confirm registration OR resend confirmation!
        .or(get_recover_route)
        .or(post_recover_route)
        .or(get_imagebrowser_route)
        .or(get_bbcodepreview_route)
        .or(post_bbcodepreview_route)
        .recover(handle_rejection)
    ).run(address).await;
}


/// 'GET':/forum is a heavily multiplexed route, since it could either be the root, the old fcid
/// threadlist, the old ftid post list, or the old fpid direct link to post
fn get_forum_route(state_filter: &BoxedFilter<(RequestContext,)>) -> BoxedFilter<(impl Reply,)> 
{
    let forum_main = warp::any()
        .and(state_filter.clone())
        .and_then(|context:RequestContext| {
            async move {
                let gc = context.global_state.clone();
                handle_response(
                    errwrap!(pages::forum_main::get_render(
                        context.into(),
                        &gc.config.forum_category_order,
                        gc.config.default_category_threads).await)?,
                    &gc.link_config
                )
            }
        }); 
    
    //struct doesn't need to escape this function!
    #[allow(dead_code)]
    #[derive(Deserialize, Debug)]
    struct FcidPage { 
        fcid: i64,
        page: Option<i32> 
    }

    let forum_fcid = warp::any()
        .and(warp::query::<FcidPage>())
        .and(state_filter.clone())
        .and_then(|fcid_page: FcidPage, context:RequestContext| {
            async move {
                let gc = context.global_state.clone();
                handle_response(
                    errwrap!(pages::forum_category::get_fcid_render(
                        context.into(),
                        fcid_page.fcid,
                        gc.config.default_display_threads, 
                        fcid_page.page).await)?,
                    &gc.link_config
                )
            }
        }); 
    
    //Don't forget to add the other stuff!
    #[allow(dead_code)]
    #[derive(Deserialize, Debug)]
    struct FtidPage { 
        ftid: i64,
        page: Option<i32> 
    }

    let forum_ftid = warp::any()
        .and(warp::query::<FtidPage>())
        .and(state_filter.clone())
        .and_then(|ftid_page: FtidPage, context:RequestContext| {
            async move {
                let gc = context.global_state.clone();
                handle_response(
                    errwrap!(pages::forum_thread::get_ftid_render(
                        context.into(),
                        ftid_page.ftid,
                        gc.config.default_display_posts, 
                        ftid_page.page).await)?,
                    &gc.link_config
                )
            }
        }); 

    #[allow(dead_code)]
    #[derive(Deserialize, Debug)]
    struct Fpid { 
        fpid: i64, 
    }

    let forum_fpid = warp::any()
        .and(warp::query::<Fpid>())
        .and(state_filter.clone())
        .and_then(|fpid: Fpid, context:RequestContext| {
            async move {
                let gc = context.global_state.clone();
                handle_response(
                    errwrap!(pages::forum_thread::get_fpid_render(
                        context.into(),
                        fpid.fpid,
                        gc.config.default_display_posts).await)?,
                    &gc.link_config
                )
            }
        }); 

    warp::get()
        .and(warp::path!("forum"))
        .and(forum_fcid.or(forum_ftid).or(forum_fpid).or(forum_main))
        .boxed()
}

/// 'POST:/login' is a multiplexed route, where multiple forms can be posted to it. We determine
/// which route to take based on the query parameter
fn post_login_multi_route(state_filter: &BoxedFilter<(RequestContext,)>, form_filter: &BoxedFilter<()>) -> 
    BoxedFilter<(impl Reply,)> 
{
    // The standard login post, main endpoint
    let login_post = warp::any()
        .and(warp::body::form::<pages::login::Login>())
        .and(state_filter.clone())
        .and_then(|form: pages::login::Login, context: RequestContext| {
            let gc = context.global_state.clone();
            let login = form.to_api_login(
                gc.config.default_cookie_expire, 
                gc.config.long_cookie_expire);
            async move {
                let (response,token) = pages::login::post_login_render(context.into(), &login).await;
                handle_response_with_token(response, &gc.link_config, token, login.expireSeconds)
            }
        }).boxed();
    
    // The secondary endpoint, to send account recovery emails
    let recover_email_post = warp::any()
        .and(qflag!(recover)) 
        .and(warp::body::form::<pages::EmailGeneric>())
        .and(state_filter.clone())
        .and_then(|_query, form: pages::EmailGeneric, context: RequestContext| {
            async move {
                let gc = context.global_state.clone();
                let response = pages::login::post_login_recover(context.into(), &form).await;
                handle_response(response, &gc.link_config)
            }
        }).boxed();

    //ALL post routes!
    warp::post()
        .and(warp::path!("login"))
        .and(form_filter.clone())
        .and(recover_email_post.or(login_post))
        .boxed()
}

/// 'POST':/register/confirm is a multiplexed route, where multiple forms can be submitted to the same endpoint.
/// These are the regular registration confirmation form (primary), and the confirmation email resend (secondary)
fn post_registerconfirm_multi_route(state_filter: &BoxedFilter<(RequestContext,)>, form_filter: &BoxedFilter<()>) -> 
    BoxedFilter<(impl Reply,)> 
{
    // Primary endpoint: finish up confirmation. Because of that, we might get a token back (on success)
    let registerconfirm_post = warp::any()
        .and(warp::body::form::<contentapi::forms::RegisterConfirm>())
        .and(state_filter.clone())
        .and_then(|form: contentapi::forms::RegisterConfirm, context: RequestContext| {
            async move {
                let gc = context.global_state.clone();
                let (response,token) = pages::registerconfirm::post_render(context.into(), &form).await;
                handle_response_with_token(response, &gc.link_config, token, gc.config.default_cookie_expire as i64)
            }
        })
        .boxed();

    // Secondary endpoint: resend confirmation email
    let registerconfirm_email_post = warp::any()
        .and(qflag!(resend)) 
        .and(warp::body::form::<pages::EmailGeneric>())
        .and(state_filter.clone())
        .and_then(|_query, form: pages::EmailGeneric, context: RequestContext| {
            async move {
                let gc = context.global_state.clone();
                let response = pages::registerconfirm::post_email_render(context.into(), &form).await;
                handle_response(response, &gc.link_config)
            }
        }).boxed();

    warp::post()
        .and(warp::path!("register"/"confirm"))
        .and(form_filter.clone())
        .and(registerconfirm_email_post.or(registerconfirm_post))
        .boxed()

}

/// 'POST':/userhome is a 3 way multiplexed route, where you can post user updates (primary), user bio updates
/// (secondary), and finally sensitive updates
fn post_userhome_multi_route(state_filter: &BoxedFilter<(RequestContext,)>, form_filter: &BoxedFilter<()>) -> 
    BoxedFilter<(impl Reply,)> 
{
    // Primary endpoint: update regular user data
    let userhome_post = warp::any()
        .and(warp::body::form::<pages::userhome::UserUpdate>())
        .and(state_filter.clone())
        .and_then(|form: pages::userhome::UserUpdate, context: RequestContext| {
            async move {
                let gc = context.global_state.clone();
                let response = errwrap!(pages::userhome::post_info_render(context.into(), form).await)?;
                handle_response(response, &gc.link_config)
            }
        })
        .boxed();

    // Secondary endpoint: user bio updates
    let userhome_bio_post = warp::any()
        .and(qflag!(bio)) 
        .and(warp::body::form::<pages::userhome::UserBio>())
        .and(state_filter.clone())
        .and_then(|_query, form: pages::userhome::UserBio, context: RequestContext| {
            async move {
                let gc = context.global_state.clone();
                let response = errwrap!(pages::userhome::post_bio_render(context.into(), form).await)?;
                handle_response(response, &gc.link_config)
            }
        }).boxed();

    // Tertiary endpoint: user sensitive updates
    let userhome_sensitive_post = warp::any()
        .and(qflag!(sensitive)) 
        .and(warp::body::form::<contentapi::forms::UserSensitive>())
        .and(state_filter.clone())
        .and_then(|_query, form: contentapi::forms::UserSensitive, context: RequestContext| {
            async move {
                let gc = context.global_state.clone();
                let response = errwrap!(pages::userhome::post_sensitive_render(context.into(), form).await)?;
                handle_response(response, &gc.link_config)
            }
        }).boxed();

    warp::post()
        .and(warp::path!("userhome"))
        .and(form_filter.clone())
        .and(userhome_bio_post.or(userhome_sensitive_post).or(userhome_post))
        .boxed()

}