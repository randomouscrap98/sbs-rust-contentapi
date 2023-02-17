use common::*;
use common::constants::{DOWNVOTE, UPVOTE, VOTETYPE};
use common::render::layout::*;
use common::submissions::get_content_vote;
use maud::*;

use contentapi::*;

pub fn render(data: MainLayoutData, content: Content, user_vote: Option<ContentEngagement>) -> String 
{
    let mut real_vote = "";

    if let Some(ref vote) = user_vote {
        if let Some(ref engagement) = vote.engagement {
            real_vote = engagement;
        }
    }

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

    basic_skeleton(&data, html! {
        title { "SmileBASIC Source Vote Widget" }
        meta name="description" content="A small widget to allow voting without reloading a main page";
        (data.links.style("/forpage/votewidget.css"))
    }, html! {
        div #"main" {
            form."nospacing" #"downvote" method="POST" action={(data.current_path)"?vote="(DOWNVOTE)} { 
                input type="submit" value="-" data-current[real_vote==DOWNVOTE];
            }
            div #"votebar" {
                div #"voteline" style=(format!("width:{}%", (upvotes as f32) / ((downvotes + upvotes) as f32) * 100.0)) { }
                div #"votecount" { ((downvotes + upvotes)) }
            }
            form."nospacing" #"upvote" method="POST" action={(data.current_path)"?vote="(UPVOTE)} { 
                input type="submit" value="+" data-current[real_vote==UPVOTE];
            }
        }
    }).into_string()
}

pub async fn get_render(context: PageContext, content_id: i64) -> Result<Response, Error>
{
    let content = context.api_context.get_content_by_id(content_id, "id,name,engagement").await?;
    let engagement = get_content_vote(&context.api_context, content_id).await?; //context.api_context.get_content_by_id(id, "id,name,engagement").await?;

    Ok(Response::Render(render(context.layout_data, content, engagement)))
}