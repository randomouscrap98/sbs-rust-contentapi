use std::{net::SocketAddr, sync::Arc};

use bbscope::BBCode;
use chrono::SecondsFormat;
use common::LinkConfig;

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
static SETTINGSCOOKIE: &str = "sbs-rust-contentapi-settings";


/// Silly thing to limit a route by a single flag present (must be i8)
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

macro_rules! std_resp {
    ($render:expr,$context:expr) => {
        async move {
            handle_response(errwrap!($render.await)?, &$context.global_state.link_config)
        }
    };
}

macro_rules! cf {
    ($ctx:ident.$setting:ident) => {
        $ctx.global_state.config.$setting
    };
}

macro_rules! gs {
    ($ctx:ident.$setting:ident) => {
        $ctx.global_state.$setting
    };
}

macro_rules! pc {
    ($ctx:ident) => {
        $ctx.page_context
    };
    ($ctx:ident.$setting:ident) => {
        $ctx.page_context.$setting
    };
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
        .and(warp::cookie::optional::<String>(SETTINGSCOOKIE))
        .and_then(move |path, method, token, config_raw| {  //Create a closure that takes ownership of map_state to let it infinitely clone
            println!("[{}] {:>5} - {:?}", chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true), &method, &path);
            let this_state = global_for_state.clone();
            async move { 
                errwrap!(RequestContext::generate(this_state, path, token, config_raw).await)
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

    let get_index_route = warp_get_async!(
        warp::path::end(), 
        |context:RequestContext| std_resp!(pages::index::get_render(pc!(context)), context)
    );

    let get_about_route = warp_get!(warp::path!("about"),
        |context:RequestContext| warp::reply::html(pages::about::render(pc!(context.layout_data))));

    let get_integrationtest_route = warp_get!(warp::path!("integrationtest"),
        |context:RequestContext| warp::reply::html(pages::integrationtest::render(pc!(context.layout_data))));

    let get_admin_route = warp_get_async!(
        warp::path!("admin"),
        |context:RequestContext| std_resp!(pages::admin::get_render(pc!(context)), context)
    );

    let get_login_route = warp_get!(warp::path!("login"),
        |context:RequestContext| warp::reply::html(pages::login::render(pc!(context.layout_data), None, None, None)));

    let get_register_route = warp_get!(warp::path!("register"),
        |context:RequestContext| warp::reply::html(pages::register::render(pc!(context.layout_data), None, None, None)));

    let get_registerconfirm_route = warp_get!(warp::path!("register"/"confirm"),
        |context:RequestContext| warp::reply::html(pages::registerconfirm::render(pc!(context.layout_data), None, None, None, None, false)));

    let get_recover_route = warp_get!(warp::path!("recover"),
        |context:RequestContext| warp::reply::html(pages::recover::render(pc!(context.layout_data), None, None)));

    let get_sessionsettings_route = warp_get!(warp::path!("sessionsettings"),
        |context:RequestContext| warp::reply::html(pages::sessionsettings::render(pc!(context.layout_data), None)));

    let get_bbcodepreview_route = warp_get!(warp::path!("widget" / "bbcodepreview"),
        |context:RequestContext| warp::reply::html(pages::widget_bbcodepreview::render(pc!(context.layout_data), &gs!(context.bbcode), None)));



    let get_logout_route = warp_get_async!(warp::path!("logout"),
        |context:RequestContext| async move {
            //Logout is a Set-Cookie to empty string with Max-Age set to 0, then redirect to root
            handle_response_with_token(
                common::Response::Redirect(String::from("/")),
                &context.global_state.link_config, 
                Some(String::from("")), 
                0
            )
        });

    let post_sessionsettings_route = warp::post()
        .and(warp::path!("sessionsettings"))
        .and(form_filter.clone())
        .and(warp::body::form::<common::UserConfig>())
        .and(state_filter.clone())
        .and_then(|form: common::UserConfig, mut context: RequestContext| {
            let mut errors: Option<Vec<String>> = None;
            let mut cookie_raw: Option<String> = None;
            match serde_json::to_string(&form) {
                Ok(cookie) => cookie_raw = Some(String::from(cookie)),
                Err(error) => errors = Some(vec![error.to_string()])
            }
            context.page_context.layout_data.user_config = form; //Is this safe? idk
            async move {
                let gc = context.global_state.clone();
                handle_response_with_anycookie(
                    common::Response::Render(pages::sessionsettings::render(context.page_context.layout_data, errors)),
                    &gc.link_config, 
                    SETTINGSCOOKIE,
                    cookie_raw,
                    gc.config.long_cookie_expire as i64
                )
            }
        })
        .boxed();

    let post_bbcodepreview_route = warp::post()
        .and(warp::path!("widget" / "bbcodepreview"))
        .and(form_filter.clone())
        .and(warp::body::form::<common::forms::BasicText>())
        .and(state_filter.clone())
        .map(|form: common::forms::BasicText, context: RequestContext| {
            warp::reply::html(pages::widget_bbcodepreview::render(context.page_context.layout_data, &context.global_state.bbcode, Some(form.text)))
        })
        .boxed();

    let get_search_route = warp_get_async!(
        warp::path!("search").and(warp::query::<common::forms::PageSearch>()),
        |search, context:RequestContext| 
            std_resp!(pages::search::get_render(pc!(context), search, cf!(context.default_display_pages)), context)
    );

    let get_activity_route = warp_get_async!(
        warp::path!("activity").and(warp::query::<pages::activity::ActivityQuery>()),
        |query, context:RequestContext| 
            std_resp!(pages::activity::get_render(pc!(context), query, cf!(context.default_activity_count)), context)
    );


    #[derive(Deserialize, Debug)]
    struct SimplePage { page: Option<i32> }

    let get_forum_category_route = warp_get_async!(
        warp::path!("forum" / "category" / String).and(warp::query::<SimplePage>()),
        |hash: String, page_struct: SimplePage, context:RequestContext| 
            std_resp!(
                pages::forum_category::get_hash_render(pc!(context), hash, cf!(context.default_display_threads), page_struct.page), 
                context
            )
    ); 

    let get_forum_thread_route = warp_get_async!(
        warp::path!("forum" / "thread" / String).and(warp::query::<SimplePage>()),
        |hash: String, page_struct: SimplePage, context:RequestContext| 
            std_resp!(
                pages::forum_thread::get_hash_render(pc!(context), hash, cf!(context.default_display_posts), page_struct.page),
                context
            )
    ); 

    let get_forum_post_route = warp_get_async!(
        warp::path!("forum" / "thread" / String / i64),
        |hash: String, post_id: i64, context:RequestContext| 
            std_resp!(
                pages::forum_thread::get_hash_postid_render(pc!(context), hash, post_id, cf!(context.default_display_posts)),
                context
            )
    ); 

    let get_user_route = warp_get_async!(
        warp::path!("user" / String),
        |username: String, context:RequestContext| 
            std_resp!(pages::user::get_render(pc!(context), username), context)
    ); 

    let get_userhome_route = warp_get_async!(
        warp::path!("userhome"),
        |context:RequestContext| 
            std_resp!(pages::userhome::get_render(pc!(context)), context)
    ); 

    let get_imagebrowser_route = warp_get_async!(
        warp::path!("widget" / "imagebrowser").and(warp::query::<pages::widget_imagebrowser::Search>()),
        |search, context:RequestContext| 
            std_resp!(
                pages::widget_imagebrowser::query_render(pc!(context), search, cf!(context.default_imagebrowser_count)),
                context
            )
    );

    let get_widgetthread_route = warp_get_async!(
        warp::path!("widget" / "thread").and(warp::query::<common::forms::ThreadQuery>()),
        |search, context:RequestContext| 
            std_resp!(pages::widget_thread::get_render(pc!(context), search), context)
    );

    let post_recover_route = warp::post()
        .and(warp::path!("recover"))
        .and(form_filter.clone())
        .and(warp::body::form::<contentapi::forms::UserSensitive>())
        .and(state_filter.clone())
        .and_then(|form: contentapi::forms::UserSensitive, context: RequestContext| {
            async move {
                let gc = context.global_state.clone();
                let (response, token) = pages::recover::post_render(pc!(context), &form).await;
                handle_response_with_token(response, &gc.link_config, token, gc.config.default_cookie_expire as i64)
            }
        }).boxed();

    let post_register_route = warp::post()
        .and(warp::path!("register"))
        .and(form_filter.clone())
        .and(warp::body::form::<contentapi::forms::Register>())
        .and(state_filter.clone())
        .and_then(|form, context: RequestContext| 
            std_resp!(pages::register::post_render(pc!(context), &form), context) 
        ).boxed();
    
    let post_thread_delete_route = warp::post()
        .and(warp::path!("forum" / "delete" / "thread" / i64))
        .and(state_filter.clone())
        .and_then(|thread_id, context: RequestContext|
            std_resp!(pages::forum_edit_thread::delete_render(pc!(context), thread_id), context)
        ).boxed();

    let post_post_delete_route = warp::post()
        .and(warp::path!("forum" / "delete" / "post" / i64))
        .and(state_filter.clone())
        .and_then(|post_id, context: RequestContext|
            std_resp!(pages::forum_edit_post::delete_render(pc!(context), post_id), context)
        ).boxed();
    
    let legacy_page_pid = warp_get_async!(
        warp::path!("page").and(warp::query::<pages::page::PageQuery>()),
        |query, context:RequestContext| 
            std_resp!(pages::page::get_pid_redirect(pc!(context), query), context)
    );
        
    warp::serve(
            fs_static_route
        .or(fs_favicon_route)
        .or(get_index_route)
        .or(get_activity_route)
        .or(get_search_route)
        .or(get_forum_route(&state_filter)) //HEAVILY multiplexed! Lots of legacy forum paths!
        .or(get_forum_edit_thread_route(&state_filter, &form_filter))
        .or(get_forum_edit_post_route(&state_filter, &form_filter))
        .or(post_thread_delete_route)
        .or(post_post_delete_route)
        .or(get_forum_category_route)
        .or(get_forum_thread_route)
        .or(get_forum_post_route)
        .or(get_about_route)
        .or(get_admin_route)
        .or(post_admin_multi_route(&state_filter, &form_filter))
        .or(get_user_route)
        .or(post_user_multi_route(&state_filter, &form_filter))
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
        .or(get_sessionsettings_route)
        .or(post_sessionsettings_route)
        .or(get_imagebrowser_route)
        .or(get_widgetthread_route)
        .or(get_bbcodepreview_route)
        .or(post_bbcodepreview_route)
        .or(legacy_page_pid)
        .or(get_integrationtest_route)
        .recover(handle_rejection)
    ).run(address).await;
}


/// 'GET':/forum is a heavily multiplexed route, since it could either be the root, the old fcid
/// threadlist, the old ftid post list, or the old fpid direct link to post
fn get_forum_route(state_filter: &BoxedFilter<(RequestContext,)>) -> BoxedFilter<(impl Reply,)> 
{
    let forum_main = warp::any()
        .and(state_filter.clone())
        .and_then(|context:RequestContext| 
            std_resp!(
                pages::forum_main::get_render(pc!(context), &cf!(context.forum_category_order), cf!(context.default_category_threads)),
                context
            )
        ); 
    
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
        .and_then(|fcid_page: FcidPage, context:RequestContext| 
            std_resp!(
                pages::forum_category::get_fcid_render(pc!(context), fcid_page.fcid, cf!(context.default_display_threads), fcid_page.page),
                context
            ) 
        ); 
    
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
        .and_then(|ftid_page: FtidPage, context:RequestContext| 
            std_resp!(
                pages::forum_thread::get_ftid_render(pc!(context), ftid_page.ftid, cf!(context.default_display_posts), ftid_page.page),
                context
            )
        ); 

    #[allow(dead_code)]
    #[derive(Deserialize, Debug)]
    struct Fpid { 
        fpid: i64, 
    }

    let forum_fpid = warp::any()
        .and(warp::query::<Fpid>())
        .and(state_filter.clone())
        .and_then(|fpid: Fpid, context:RequestContext| 
            std_resp!(
                pages::forum_thread::get_fpid_render(pc!(context), fpid.fpid, cf!(context.default_display_posts)),
                context
            )
        ); 

    warp::get()
        .and(warp::path!("forum"))
        .and(forum_fcid.or(forum_ftid).or(forum_fpid).or(forum_main))
        .boxed()
}

fn get_forum_edit_thread_route(state_filter: &BoxedFilter<(RequestContext,)>, form_filter: &BoxedFilter<()>) -> BoxedFilter<(impl Reply,)> 
{
    //struct doesn't need to escape this function!
    #[allow(dead_code)]
    #[derive(Deserialize, Debug)]
    struct CategoryParameter { 
        category: String
    }

    let thread_new = warp::any()
        .and(warp::query::<CategoryParameter>())
        .and(state_filter.clone())
        .and_then(|catparam: CategoryParameter, context:RequestContext| 
            std_resp!(
                pages::forum_edit_thread::get_render(pc!(context), Some(catparam.category), None),
                context
            ) 
        ).boxed(); 
    
    //Don't forget to add the other stuff!
    #[allow(dead_code)]
    #[derive(Deserialize, Debug)]
    struct ThreadParameter { 
        thread: String
    }

    let thread_edit = warp::any()
        .and(warp::query::<ThreadParameter>())
        .and(state_filter.clone())
        .and_then(|threadparam: ThreadParameter, context:RequestContext| 
            std_resp!(
                pages::forum_edit_thread::get_render(pc!(context), None, Some(threadparam.thread)),
                context
            )
        ).boxed(); 

    let thread_post = warp::any()
        .and(warp::body::form::<common::forms::ThreadForm>())
        .and(state_filter.clone())
        .and_then(|form: common::forms::ThreadForm, context: RequestContext| {
            std_resp!(pages::forum_edit_thread::post_render(pc!(context), form), context) 
        }).boxed();

    warp::path!("forum" / "edit" / "thread")
        .and(warp::get().and(thread_new.or(thread_edit))
            .or(warp::post().and(form_filter.clone()).and(thread_post)))
        .boxed()
}

fn get_forum_edit_post_route(state_filter: &BoxedFilter<(RequestContext,)>, form_filter: &BoxedFilter<()>) -> BoxedFilter<(impl Reply,)> 
{
    //struct doesn't need to escape this function!
    #[allow(dead_code)]
    #[derive(Deserialize, Debug)]
    struct NewPostParameters { 
        thread: String,
        reply: Option<i64>
    }

    let post_new = warp::any()
        .and(warp::query::<NewPostParameters>())
        .and(state_filter.clone())
        .and_then(|param: NewPostParameters, context:RequestContext| 
            std_resp!(
                pages::forum_edit_post::get_render(pc!(context), Some(param.thread), None, param.reply),
                context
            ) 
        ).boxed(); 
    
    //Don't forget to add the other stuff!
    #[allow(dead_code)]
    #[derive(Deserialize, Debug)]
    struct EditPostParameter { 
        post: i64
    }

    let post_edit = warp::any()
        .and(warp::query::<EditPostParameter>())
        .and(state_filter.clone())
        .and_then(|param: EditPostParameter, context:RequestContext| 
            std_resp!(
                pages::forum_edit_post::get_render(pc!(context), None, Some(param.post), None),
                context
            )
        ).boxed(); 

    let post_post = warp::any()
        .and(warp::body::form::<common::forms::PostForm>())
        .and(state_filter.clone())
        .and_then(|form: common::forms::PostForm, context: RequestContext| {
            std_resp!(pages::forum_edit_post::post_render(pc!(context), form), context) 
        }).boxed();

    warp::path!("forum" / "edit" / "post")
        .and(warp::get().and(post_new.or(post_edit))
            .or(warp::post().and(form_filter.clone()).and(post_post)))
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
                let (response,token) = pages::login::post_login_render(pc!(context), &login).await;
                handle_response_with_token(response, &gc.link_config, token, login.expireSeconds)
            }
        }).boxed();
    
    // The secondary endpoint, to send account recovery emails
    let recover_email_post = warp::any()
        .and(qflag!(recover)) 
        .and(warp::body::form::<common::forms::EmailGeneric>())
        .and(state_filter.clone())
        .and_then(|_query, form: common::forms::EmailGeneric, context: RequestContext| {
            async move {
                let gc = context.global_state.clone();
                let response = pages::login::post_login_recover(pc!(context), &form).await;
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
        .and_then(|form, context: RequestContext| {
            async move {
                let gc = context.global_state.clone();
                let (response,token) = pages::registerconfirm::post_render(pc!(context), &form).await;
                handle_response_with_token(response, &gc.link_config, token, gc.config.default_cookie_expire as i64)
            }
        })
        .boxed();

    // Secondary endpoint: resend confirmation email
    let registerconfirm_email_post = warp::any()
        .and(qflag!(resend)) 
        .and(warp::body::form::<common::forms::EmailGeneric>())
        .and(state_filter.clone())
        .and_then(|_query, form: common::forms::EmailGeneric, context: RequestContext| 
            std_resp!(pages::registerconfirm::post_email_render(pc!(context), &form), context)
        ).boxed();

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
        .and_then(|form, context: RequestContext| 
            std_resp!(pages::userhome::post_info_render(pc!(context), form), context)
        ).boxed();

    // Secondary endpoint: user bio updates
    let userhome_bio_post = warp::any()
        .and(qflag!(bio)) 
        .and(warp::body::form::<common::forms::BasicPage>())
        .and(state_filter.clone())
        .and_then(|_query, form, context: RequestContext| 
            std_resp!(pages::userhome::post_bio_render(pc!(context), form), context)
        ).boxed();

    // Tertiary endpoint: user sensitive updates
    let userhome_sensitive_post = warp::any()
        .and(qflag!(sensitive)) 
        .and(warp::body::form::<contentapi::forms::UserSensitive>())
        .and(state_filter.clone())
        .and_then(|_query, form, context: RequestContext| 
            std_resp!(pages::userhome::post_sensitive_render(pc!(context), form), context) 
        ).boxed();

    warp::post()
        .and(warp::path!("userhome"))
        .and(form_filter.clone())
        .and(userhome_bio_post.or(userhome_sensitive_post).or(userhome_post))
        .boxed()

}

fn post_admin_multi_route(state_filter: &BoxedFilter<(RequestContext,)>, form_filter: &BoxedFilter<()>) -> 
    BoxedFilter<(impl Reply,)> 
{
    let admin_registrationconfig_post = warp::any()
        .and(qflag!(registrationconfig)) 
        //For now, we use the direct form
        .and(warp::body::form::<contentapi::forms::RegistrationConfig>())
        .and(state_filter.clone())
        .and_then(|_query, form, context: RequestContext| 
            std_resp!(pages::admin::post_registrationconfig(pc!(context), form), context)
        ).boxed();

    let admin_frontpage_post = warp::any()
        .and(qflag!(frontpage)) 
        .and(warp::body::form::<common::forms::BasicPage>())
        .and(state_filter.clone())
        .and_then(|_query, form, context: RequestContext| 
            std_resp!(pages::admin::post_frontpage(pc!(context), form), context)
        ).boxed();

    let admin_alert_post = warp::any()
        .and(qflag!(alert)) 
        .and(warp::body::form::<common::forms::BasicPage>())
        .and(state_filter.clone())
        .and_then(|_query, form, context: RequestContext| 
            std_resp!(pages::admin::post_alert(pc!(context), form), context)
        ).boxed();

    warp::post()
        .and(warp::path!("admin"))
        .and(form_filter.clone())
        .and(admin_registrationconfig_post.or(admin_frontpage_post).or(admin_alert_post))
        .boxed()

}

fn post_user_multi_route(state_filter: &BoxedFilter<(RequestContext,)>, form_filter: &BoxedFilter<()>) -> 
    BoxedFilter<(impl Reply,)> 
{
    let base_route = warp::post().and(warp::path!("user" / String)).and(form_filter.clone());

    let user_ban_route = base_route.clone()
        .and(qflag!(ban)) 
        .and(warp::body::form::<common::forms::BanForm>())
        .and(state_filter.clone())
        .and_then(|username, _query, form, context: RequestContext| 
            std_resp!(pages::user::post_ban(pc!(context), username, form), context)
        ).boxed();

    let user_unban_route = base_route.clone()
        .and(qflag!(unban)) 
        .and(warp::body::form::<common::forms::UnbanForm>())
        .and(state_filter.clone())
        .and_then(|username, _query, form, context: RequestContext| 
            std_resp!(pages::user::post_unban(pc!(context), username, form), context)
        ).boxed();

    user_ban_route.or(user_unban_route).boxed()
}