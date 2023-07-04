
use common::render::submissions::pageicon_limited;
use common::{*, constants::ACTIVITYTYPES};
use common::render::layout::*;
use common::response::*;
use contentapi::*;
use maud::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct RecentActivityConfig {
    pub count: Option<i32>
}

impl Default for RecentActivityConfig {
    fn default() -> Self {
        Self { 
            count: None 
        }
    }
}

/// The full rendering code. 
pub async fn get_render(mut context: PageContext, config: RecentActivityConfig) -> Result<Response, Error>
{
    //const BYREVISIONREQUEST : &str = "by_revision";
    //const BYCOMMENTREQUEST : &str = "by_comment";

    let data = &context.layout_data;
    let count = if let Some(c) = config.count { c } else { 20 };

    let mut request = FullRequest::new();
    add_value!(request, "allowed_types", ACTIVITYTYPES);

    let content_request = build_request!(
        RequestType::content,
        String::from("id,name,description,values,lastActionDate,hash,literalType,contentType"), 
        String::from("literalType in @allowed_types"),
        String::from("lastActionDate_desc"),
        count
    );
    request.requests.push(content_request);

    let result = context.api_context.post_request_profiled_opt(&request, "recentactivity").await?;
    let content = conversion::cast_result_required::<Content>(&result, &RequestType::content.to_string())?;

    //content_revision_request.name = Some(String::from(""));

    Ok(Response::Render(
        basic_skeleton(data, html! {
            title { "SmileBASIC Source Recent Activity" }
            meta name="description" content="A list of recent content";
            (data.links.style("/forpage/forum.css")) //Kind of a big thing to include but it's just easier
            (data.links.style("/forpage/recentactivity.css"))
        }, html! {
            @for ref c in content {
                div."smallseparate recentactivityitem" { //."recentactivityitem" { 
                    span."threadicon" /* ."pageicon"*/ { (pageicon_limited(&data.links, c, 1)) }
                    a."pagetitle flatlink" target="_top" href=(data.links.forum_thread(c)) { (opt_s!(c.name)) }
                }
            }
        }).into_string()
    ))
}
