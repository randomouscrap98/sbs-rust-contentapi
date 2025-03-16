use common::capi::get_msgid_by_cid;
use common::*;
use common::{capi::get_basic_by_pid, response::*};
//use contentapi::conversion::*;
//use contentapi::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(default)]
pub struct PageQuery {
    pid: i64,
    cid: Option<i64>,
}

//https://old.smilebasicsource.com/page?pid=1497&cid=16922#comment_16922
pub async fn get_pid_redirect(context: PageContext, query: PageQuery) -> Result<Response, Error> {
    let page_data = get_basic_by_pid(&context, query.pid)?
        .ok_or(Error::NotFound(String::from("Could not find page!")))?;

    if let Some(cid) = query.cid {
        if let Some(msgid) = get_msgid_by_cid(&context, page_data.id, cid)? {
            //println!("FOUND MSGID: {}", msgid);
            let url = context
                .layout_data
                .links
                .forum_post_unsafe(msgid, &page_data.hash);
            return Ok(Response::Redirect(url));
        }
    }

    let url = context
        .layout_data
        .links
        .forum_thread_unsafe(&page_data.hash);
    return Ok(Response::Redirect(url));
}
