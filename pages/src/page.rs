//use super::*;
use contentapi::*;
use contentapi::conversion::*;
use common::*;
use common::response::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(default)]
pub struct PageQuery { 
    pid: i64,
    cid: Option<i64>
    //page: Option<i32> 
}

//https://old.smilebasicsource.com/page?pid=1497&cid=16922#comment_16922
pub async fn get_pid_redirect(mut context: PageContext, query: PageQuery) -> Result<Response, Error>
{
    let mut request = FullRequest::new();
    add_value!(request, "pidkey", vec!["pid"]);
    add_value!(request, "pid", vec![query.pid]);

    //Basically: go look for the content that has the given pid
    let pid_request = build_request!(
        RequestType::content,
        String::from("id,hash,values"),
        format!("!valuein(@pidkey, @pid)")
    );
    request.requests.push(pid_request);

    if let Some(cid) = query.cid {
        add_value!(request, "cidkey", vec!["cid"]);
        add_value!(request, "cid", vec![cid]);
        let cid_request = build_request!(
            RequestType::message,
            String::from("id,values,contentId"),
            format!("!valuein(@cidkey, @cid)")
        );
        request.requests.push(cid_request);
    }

    let result = context.api_context.post_request_profiled_opt(&request, "legacy_page").await?;
    let mut pages = cast_result_required::<Content>(&result, "content")?;
    let mut messages = cast_result_safe::<Message>(&result, "message")?;

    let page = pages.pop().ok_or(Error::NotFound(String::from("Could not find page!")))?;

    let mut url = format!("/forum/thread/{}", opt_s!(&page.hash));

    if let Some(message) = messages.pop() {
        if let Some(id) = message.id {
            url = format!("{}/{}{}", url, id, LinkConfig::forum_post_hash(&message));
        }
    }

    Ok(Response::Redirect(url))
}