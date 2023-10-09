use axum::extract::Query;

use crate::state::RequestContext;

use super::StdResponse;

#[allow(dead_code)]
#[derive(serde::Deserialize, Debug)]
pub struct ForumFullQuery { 
    fcid: Option<i64>, //Lots of options because of legacy junk
    ftid: Option<i64>,
    fpid: Option<i64>,
    page: Option<i32> 
}

pub async fn forum_get(context: RequestContext, Query(query): Query<ForumFullQuery>) -> StdResponse
{
    //Order goes from most precise to least
    if let Some(fpid) = query.fpid {
        pages::forum_thread::get_fpid_render(context.page_context, fpid, 
            context.global_state.config.default_display_posts).await
    }
    else if let Some(ftid) = query.ftid {
        pages::forum_thread::get_ftid_render(context.page_context, ftid, context.global_state.config.default_display_posts, query.page).await
    }
    else if let Some(fcid) = query.fcid {
        //Err(common::response::Error::NotFound(String::from("FCID is disabled right now")))
        pages::forum_category::get_fcid_render(context.page_context, fcid, context.global_state.config.default_display_threads, query.page).await
    }
    else {
        //This is main forum display, usually what we want (but unfortunately last)
        pages::forum_main::get_render(context.page_context, 
            &context.global_state.config.forum_category_order, context.global_state.config.default_category_threads).await
    }
}

/// Existence of parameters indicates which kind of form to generate
#[derive(serde::Deserialize, Debug)]
pub struct ThreadEditParameter { 
    pub category: Option<String>,
    pub thread: Option<String>
}

/// Existence of parameters indicates which kind of form to generate
#[derive(serde::Deserialize, Debug)]
pub struct PostEditParameters { 
    pub post: Option<i64>,          // Either post is set...
    pub thread: Option<String>,     // Or thread is set. But we don't worry about it, because the logic is in the forum renderer, not our router
    pub reply: Option<i64>,
    pub widget: Option<bool>
}
