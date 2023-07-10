use serde::Deserialize;
use warp::*;
use warp::filters::*;
//use warp::Filter::*;
//use warp::filters::BoxedFilter;
//use warp::{Filter, Reply};

use crate::state::*;
use crate::generic_handlers::*;
use crate::*;

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

