use common::constants::{DOWNVOTE, UPVOTE, VOTETYPE};
use common::render::layout::*;
use common::response::*;
use common::*;
use maud::*;

use contentapi::*;

pub fn render(data: MainLayoutData, content: Content) -> String {
    let mut downvotes = 0;
    let mut upvotes = 0;

    if let Some(ref engagement) = content.engagement {
        if let Some(vote_engagements) = engagement.get(VOTETYPE) {
            if let Some(downvote_count) = vote_engagements.get(DOWNVOTE) {
                downvotes = *downvote_count;
            }
            if let Some(upvote_count) = vote_engagements.get(UPVOTE) {
                upvotes = *upvote_count;
            }
        }
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
    let content = context
        .api_context
        .get_content_by_id(content_id, "id,name,engagement")
        .await?;

    return Ok(Response::Render(render(context.layout_data, content)));
}
