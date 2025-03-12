use std::collections::HashMap;

use common::capi::get_engagements;
use common::constants::{DOWNVOTESTR, UPVOTESTR};
use common::render::layout::*;
use common::response::*;
use common::*;
use maud::*;

pub fn render(data: MainLayoutData, engagement: HashMap<String, i32>) -> String {
    let mut downvotes = 0;
    let mut upvotes = 0;

    if let Some(downvote_count) = engagement.get(DOWNVOTESTR) {
        downvotes = *downvote_count;
    }
    if let Some(upvote_count) = engagement.get(UPVOTESTR) {
        upvotes = *upvote_count;
    }

    let totalvotes = downvotes + upvotes;

    basic_skeleton(&data, html! {
        title { "SmileBASIC Source Vote Widget" }
        meta name="description" content="A small widget to allow voting without reloading a main page";
        (data.links.style("/forpage/votewidget.css"))
    }, html! {
        div #"main" {
            div #"votebar" data-votes=(totalvotes) {
                div #"voteline" style=(format!("width:{}%", (upvotes as f32) / (totalvotes as f32) * 100.0)) { }
                div #"votecount" { (totalvotes) }
            }
        }
    }).into_string()
}

pub async fn get_render(context: PageContext, content_id: i64) -> Result<Response, Error> {
    let engagement = get_engagements(&context, content_id)?;
    return Ok(Response::Render(render(context.layout_data, engagement)));
}
