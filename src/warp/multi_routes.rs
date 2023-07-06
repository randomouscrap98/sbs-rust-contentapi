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

