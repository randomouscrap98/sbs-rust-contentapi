use common::*;
use common::prefab::*;
use common::constants::{DOWNVOTE, UPVOTE, VOTETYPE};
use common::forms::VoteForm;
use common::render::layout::*;
use common::response::*;
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

    let totalvotes = downvotes + upvotes;

    basic_skeleton(&data, html! {
        title { "SmileBASIC Source Vote Widget" }
        meta name="description" content="A small widget to allow voting without reloading a main page";
        (data.links.style("/forpage/votewidget.css"))
    }, html! {
        div #"main" {
            @if data.user.is_some() {
                form."nospacing" #"downvote" method="POST" action=(data.current()) { 
                    input type="hidden" name="vote" value=(DOWNVOTE);
                    input."notheme" type="submit" value="-" title="Downvote" data-current[real_vote==DOWNVOTE];
                }
            }
            div #"votebar" data-votes=(totalvotes) {
                div #"voteline" style=(format!("width:{}%", (upvotes as f32) / (totalvotes as f32) * 100.0)) { }
                div #"votecount" { (totalvotes) }
            }
            @if data.user.is_some() {
                form."nospacing" #"upvote" method="POST" action=(data.current()) { 
                    input type="hidden" name="vote" value=(UPVOTE);
                    input."notheme" type="submit" value="+" title="Upvote" data-current[real_vote==UPVOTE];
                }
            }
        }
    }).into_string()
}

//fn get_render_base(context: PageContext, content_id: i64) -> Result<Response, Error>
//{
//
//}

pub async fn get_render(context: PageContext, content_id: i64) -> Result<Response, Error>
{
    let content = context.api_context.get_content_by_id(content_id, "id,name,engagement").await?;
    let engagement = get_content_vote(&context.api_context, content_id).await?; //context.api_context.get_content_by_id(id, "id,name,engagement").await?;

    Ok(Response::Render(render(context.layout_data, content, engagement)))
}

pub async fn post_render(context: PageContext, content_id: i64, form: VoteForm) -> Result<Response, Error>
{
    context.api_context.post_set_content_engagement(content_id, VOTETYPE, &form.vote).await?;
    get_render(context, content_id).await
}