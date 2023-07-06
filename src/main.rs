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

    ////This "state filter" should be placed at the end of your path but before you start collecting your
    ////route-specific data. It will collect the path and the session cookie (if there is one) and create
    ////a context with lots of useful data to pass to all the templates (but not ALL of it like before)
    //let global_for_state = global_state.clone();
    //let state_filter = warp::path::full()
    //    .and(warp::method())
    //    .and(warp::cookie::optional::<String>(SESSIONCOOKIE))
    //    .and(warp::cookie::optional::<String>(SETTINGSCOOKIE))
    //    .and_then(move |path, method, token, config_raw| {  //Create a closure that takes ownership of map_state to let it infinitely clone
    //        println!("[{}] {:>5} - {:?}", chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true), &method, &path);
    //        let this_state = global_for_state.clone();
    //        async move { 
    //            errwrap!(RequestContext::generate(this_state, path, token, config_raw).await)
    //        }
    //    }).boxed();
    
    //let global_for_form = global_state.clone();
    //let form_filter = warp::body::content_length_limit(global_for_form.config.body_maxsize as u64).boxed();

    //#[derive(Deserialize, Debug)]
    //struct SimplePage { page: Option<i32> }

    //let get_imagebrowser_route = warp_get_async!(
    //    warp::path!("widget" / "imagebrowser").and(warp::query::<pages::widget_imagebrowser::Search>()),
    //    |search, context:RequestContext| 
    //        std_resp!(
    //            pages::widget_imagebrowser::query_render(pc!(context), search, cf!(context.default_imagebrowser_count)),
    //            context
    //        )
    //);

    //let get_widgetthread_route = warp_get_async!(
    //    warp::path!("widget" / "thread").and(warp::query::<common::forms::ThreadQuery>()),
    //    |search, context:RequestContext| 
    //        std_resp!(pages::widget_thread::get_render(pc!(context), search), context)
    //);

    //let get_votewidget_route = warp_get_async!(
    //    warp::path!("widget" / "votes" / i64),
    //    |content_id, context:RequestContext| 
    //        std_resp!(pages::widget_votes::get_render(pc!(context), content_id), context)
    //);

    //let get_recentactivity_route = warp_get_async!(
    //    warp::path!("widget" / "recentactivity").and(warp::query::<pages::widget_recentactivity::RecentActivityConfig>()),
    //    |query, context:RequestContext| 
    //        std_resp!(pages::widget_recentactivity::get_render(pc!(context), query), context)
    //);

    //#[derive(Deserialize, Default)]
    //struct QrParam {
    //    high_density: Option<bool>
    //}

    //let get_qrwidget_route = warp_get_async!(
    //    warp::path!("widget" / "qr" / String).and(warp::query::<QrParam>()),
    //    |hash: String, qr_param : QrParam, context:RequestContext| 
    //        std_resp!(pages::widget_qr::get_render(pc!(context), &hash, 
    //            if let Some(hd) = qr_param.high_density { hd } else { false }), context)
    //);

    //let post_votewidget_route = warp::post()
    //    .and(warp::path!("widget" / "votes" / i64))
    //    .and(form_filter.clone())
    //    .and(warp::body::form::<common::forms::VoteForm>())
    //    .and(state_filter.clone())
    //    .and_then(|content_id, form, context: RequestContext|
    //        std_resp!(pages::widget_votes::post_render(pc!(context), content_id, form), context)
    //    ).boxed();

    //warp::serve(
    //        fs_static_route
    //    .or(fs_favicon_route)
    //    .or(fs_robots_route)
    //    .or(get_index_route)
    //    .or(get_about_route)
    //    .or(get_search_route)
    //    .or(get_searchall_route)
    //    .or(get_admin_route)
    //    .or(get_documentation_route)
    //    .or(post_admin_multi_route(&state_filter, &form_filter))
    //    .or(get_activity_route)
    //        .boxed()
    //    .or(get_forum_route(&state_filter)) //HEAVILY multiplexed! Lots of legacy forum paths!
    //    .or(get_forum_edit_thread_route(&state_filter, &form_filter))
    //    .or(get_forum_edit_post_route(&state_filter, &form_filter))
    //    .or(get_page_edit_route(&state_filter, &form_filter))
    //    .or(post_thread_delete_route)
    //    .or(post_post_delete_route)
    //    .or(post_page_delete_route)
    //    .or(get_forum_category_route)
    //    .or(get_forum_thread_route)
    //    .or(get_forum_post_route)
    //        .boxed()
    //    .or(get_user_route)
    //    .or(post_user_multi_route(&state_filter, &form_filter))
    //    .or(get_userhome_route)
    //    .or(post_userhome_multi_route(&state_filter, &form_filter)) //Multiplexed! Login OR send recovery!
    //    .or(get_login_route)
    //    .or(post_login_multi_route(&state_filter, &form_filter)) //Multiplexed! Login OR send recovery!
    //    .or(get_logout_route)
    //    .or(get_register_route)
    //    .or(post_register_route)
    //    .or(get_registerconfirm_route)
    //    .or(post_registerconfirm_multi_route(&state_filter, &form_filter)) //Multiplexed! Confirm registration OR resend confirmation!
    //    .or(get_recover_route)
    //    .or(post_recover_route)
    //    .or(get_sessionsettings_route)
    //    .or(post_sessionsettings_route)
    //        .boxed()
    //    .or(get_imagebrowser_route)
    //    .or(get_widgetthread_route)
    //    .or(get_votewidget_route)
    //    .or(post_votewidget_route)
    //    .or(get_bbcodepreview_route)
    //    .or(post_contentpreview_route)
    //    .or(get_qrwidget_route)
    //    .or(get_recentactivity_route)
    //    .or(post_bbcodepreview_route)
    //    .or(legacy_page_pid)
    //    .or(get_integrationtest_route)
    //    .recover(handle_rejection)
    //).run(address).await;
}

