use maud::*;
use std::collections::HashMap;

use super::common::contentapi::*;
use super::context::*;
use crate::layout::*;
use crate::response::*;

fn get_engagements(ctx: &PageContext, content: i64) -> Result<HashMap<String, i32>, Error> {
    // WARN: you can get engagements for content that is private! You also get engagements
    // for deleted users/etc!
    let query = format!("SELECT `type`,engagement,count(*) FROM content_engagement WHERE contentId = ? GROUP BY `type`,engagement");
    let mut stmt = ctx.dbcon.prepare(&query)?;
    let engagement_iter = stmt.query_map([&content], |row| {
        let mut typ: String = row.get(0)?;
        let eng: String = row.get(1)?;
        typ.push_str(&eng);
        let count: i32 = row.get(2)?;
        Ok((typ, count))
    })?;

    #[cfg(feature = "querydump")]
    println!("Query: {:?}", &query);

    let mut result = HashMap::<String, i32>::new();

    for engagement in engagement_iter {
        let reng = engagement?;
        result.insert(reng.0, reng.1);
    }

    Ok(result)
}

fn render(data: MainLayoutData, engagement: HashMap<String, i32>) -> String {
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
