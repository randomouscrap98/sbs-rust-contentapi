use serde::Deserialize;
use warp::*;
use warp::filters::*;
//use warp::Filter::*;
//use warp::filters::BoxedFilter;
//use warp::{Filter, Reply};

use crate::state::*;
use crate::generic_handlers::*;
use crate::*;

/// 'GET':/forum is a heavily multiplexed route, since it could either be the root, the old fcid
/// threadlist, the old ftid post list, or the old fpid direct link to post
pub fn get_forum_route(state_filter: &BoxedFilter<(RequestContext,)>) -> BoxedFilter<(impl Reply,)> 
{
    let forum_main = warp::any()
        .and(state_filter.clone())
        .and_then(|context:RequestContext| 
            std_resp!(
                pages::forum_main::get_render(pc!(context), &cf!(context.forum_category_order), cf!(context.default_category_threads)),
                context
            )
        ).boxed(); 
    
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
        ).boxed(); 
    
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
        ).boxed(); 

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
        ).boxed(); 

    warp::get()
        .and(warp::path!("forum"))
        .and(forum_fcid.or(forum_ftid).or(forum_fpid).or(forum_main))
        .boxed()
}

pub fn get_forum_edit_thread_route(state_filter: &BoxedFilter<(RequestContext,)>, form_filter: &BoxedFilter<()>) -> BoxedFilter<(impl Reply,)> 
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

pub fn get_forum_edit_post_route(state_filter: &BoxedFilter<(RequestContext,)>, form_filter: &BoxedFilter<()>) -> BoxedFilter<(impl Reply,)> 
{
    //struct doesn't need to escape this function!
    #[allow(dead_code)]
    #[derive(Deserialize, Debug)]
    struct NewPostParameters { 
        thread: String,
        reply: Option<i64>,
        widget: Option<bool>
    }

    let post_new = warp::any()
        .and(warp::query::<NewPostParameters>())
        .and(state_filter.clone())
        .and_then(|param: NewPostParameters, context:RequestContext| 
            std_resp!(
                pages::forum_edit_post::get_render(pc!(context), Some(param.thread), None, param.reply, 
                    if let Some(wid) = param.widget {wid} else { false }),
                context
            ) 
        ).boxed(); 
    
    //Don't forget to add the other stuff!
    #[allow(dead_code)]
    #[derive(Deserialize, Debug)]
    struct EditPostParameter { 
        post: i64,
        widget: Option<bool>
    }

    let post_edit = warp::any()
        .and(warp::query::<EditPostParameter>())
        .and(state_filter.clone())
        .and_then(|param: EditPostParameter, context:RequestContext| 
            std_resp!(
                pages::forum_edit_post::get_render(pc!(context), None, Some(param.post), None, 
                    if let Some(wid) = param.widget {wid} else { false }),
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

pub fn get_page_edit_route(state_filter: &BoxedFilter<(RequestContext,)>, form_filter: &BoxedFilter<()>) -> BoxedFilter<(impl Reply,)> 
{
    //struct doesn't need to escape this function!
    #[allow(dead_code)]
    #[derive(Deserialize, Debug)]
    struct SubtypeParameter { 
        mode: String
    }

    let page_new = warp::any()
        .and(warp::query::<SubtypeParameter>())
        .and(state_filter.clone())
        .and_then(|param: SubtypeParameter, context:RequestContext| 
            std_resp!(
                pages::page_edit::get_render(pc!(context), Some(param.mode), None),
                context
            ) 
        ).boxed(); 
    
    //Don't forget to add the other stuff!
    #[allow(dead_code)]
    #[derive(Deserialize, Debug)]
    struct PageParameter { 
        page: String
    }

    let page_edit = warp::any()
        .and(warp::query::<PageParameter>())
        .and(state_filter.clone())
        .and_then(|param: PageParameter, context:RequestContext| 
            std_resp!(
                pages::page_edit::get_render(pc!(context), None, Some(param.page)),
                context
            )
        ).boxed(); 

    let page_post = warp::any()
        .and(warp::body::form::<common::forms::PageForm>())
        .and(state_filter.clone())
        .and_then(|form: common::forms::PageForm, context: RequestContext| {
            std_resp!(pages::page_edit::post_render(pc!(context), form), context) 
        }).boxed();

    warp::path!("page" / "edit")
        .and(warp::get().and(page_new.or(page_edit))
            .or(warp::post().and(form_filter.clone()).and(page_post)))
        .boxed()
}

/// 'POST:/login' is a multiplexed route, where multiple forms can be posted to it. We determine
/// which route to take based on the query parameter
pub fn post_login_multi_route(state_filter: &BoxedFilter<(RequestContext,)>, form_filter: &BoxedFilter<()>) -> 
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
pub fn post_registerconfirm_multi_route(state_filter: &BoxedFilter<(RequestContext,)>, form_filter: &BoxedFilter<()>) -> 
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
pub fn post_userhome_multi_route(state_filter: &BoxedFilter<(RequestContext,)>, form_filter: &BoxedFilter<()>) -> 
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

pub fn post_admin_multi_route(state_filter: &BoxedFilter<(RequestContext,)>, form_filter: &BoxedFilter<()>) -> 
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

    let admin_docscustom_post = warp::any()
        .and(qflag!(docscustom)) 
        .and(warp::body::form::<common::forms::BasicPage>())
        .and(state_filter.clone())
        .and_then(|_query, form, context: RequestContext| 
            std_resp!(pages::admin::post_docscustom(pc!(context), form), context)
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
        .and(admin_registrationconfig_post.or(admin_frontpage_post).or(admin_alert_post).or(admin_docscustom_post))
        .boxed()

}

pub fn post_user_multi_route(state_filter: &BoxedFilter<(RequestContext,)>, form_filter: &BoxedFilter<()>) -> 
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