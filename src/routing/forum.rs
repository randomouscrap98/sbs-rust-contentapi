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
        pages::forum_category::get_fcid_render(context.page_context, fcid, context.global_state.config.default_display_threads, query.page).await
    }
    else {
        //This is main forum display, usually what we want (but unfortunately last)
        pages::forum_main::get_render(context.page_context, 
            &context.global_state.config.forum_category_order, context.global_state.config.default_category_threads).await
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct ThreadEditParameter { 
    pub category: Option<String>,
    pub thread: Option<String>
}