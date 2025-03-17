//use common::capi::get_msgid_by_cid;
//use common::*;
//use common::{capi::get_basic_by_pid, response::*};
//use contentapi::conversion::*;
//use contentapi::*;
use serde::{Deserialize, Serialize};

use crate::pages::common::*;
use crate::pages::context::*;
use crate::response::*;

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(default)]
pub struct PageQuery {
    pid: i64,
    cid: Option<i64>,
}

pub fn get_basic_by_pid(ctx: &PageContext, pid: i64) -> Result<Option<BasicContent>, Error> {
    let query = format!(
        "SELECT {} FROM content c {} JOIN content_values v ON v.contentId=c.id WHERE  v.`key`=? AND v.value=? GROUP BY c.id",
        BASICCONTENTFIELDS, join_common_content("c.id")
    );
    let mut stmt = ctx.dbcon.prepare(&query)?;
    let spid = format!("{}", pid);
    Ok(gather_basiccontent((&mut stmt, &query), rusqlite::params!["pid", &spid])?.pop())
}

pub fn get_msgid_by_cid(
    ctx: &PageContext,
    content_id: i64,
    cid: i64,
) -> Result<Option<i64>, Error> {
    let query = format!(
        "SELECT m.id FROM messages m JOIN message_values v ON m.id=v.messageId WHERE v.`key`=? AND v.`value`=? AND m.contentId = ?",
    );
    let scid = format!("{}", cid);
    let scontent_id = format!("{}", content_id);
    let mut stmt = ctx.dbcon.prepare(&query)?;
    Ok(easy_query(
        (&mut stmt, &query),
        rusqlite::params!["cid", &scid, &scontent_id],
        |row| row.get::<usize, i64>(0),
    )?
    .pop())
}

pub fn get_msgid_contenthash_by_fpid(
    ctx: &PageContext,
    fpid: i64,
) -> Result<Option<(i64, String)>, Error> {
    let query = format!(
        "SELECT m.id,(SELECT c.hash FROM content c {} WHERE m.contentId = c.id) FROM messages m JOIN message_values v ON m.id=v.messageId WHERE v.`key`=? AND v.`value`=?",
        join_common_content("c.id")
    );
    let sfpid = format!("{}", fpid);
    let mut stmt = ctx.dbcon.prepare(&query)?;
    Ok(easy_query(
        (&mut stmt, &query),
        rusqlite::params!["fpid", &sfpid],
        |row| Ok((row.get::<usize, i64>(0)?, row.get::<usize, String>(1)?)),
    )?
    .pop())
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

pub fn get_contenthash_by_ftid(ctx: &PageContext, ftid: i64) -> Result<Option<String>, Error> {
    let query = format!(
        "SELECT c.hash FROM content c {} JOIN content_values v ON v.contentId = c.id WHERE v.`key`=? AND v.`value`=?",
        join_common_content("c.id")
    );
    let sftid = format!("{}", ftid);
    let mut stmt = ctx.dbcon.prepare(&query)?;
    Ok(easy_query(
        (&mut stmt, &query),
        rusqlite::params!["ftid", &sftid],
        |row| row.get::<usize, String>(0),
    )?
    .pop())
}

pub async fn get_ftid_redirect(
    context: PageContext,
    ftid: i64,
    page: Option<i32>,
) -> Result<Response, Error> {
    if let Some(hash) = get_contenthash_by_ftid(&context, ftid)? {
        let mut url = context.layout_data.links.forum_thread_unsafe(&hash);
        if let Some(page) = page {
            url.push_str(&format!("?page={}", page))
        }
        Ok(Response::Redirect(url))
    } else {
        Err(Error::NotFound(format!("Can't find thread {}", ftid)))
    }
}

//Most old links may be to posts directly? idk
pub async fn get_fpid_redirect(context: PageContext, fpid: i64) -> Result<Response, Error> {
    // These are redirects now, 2025
    if let Some((msgid, hash)) = get_msgid_contenthash_by_fpid(&context, fpid)? {
        let url = context.layout_data.links.forum_post_unsafe(msgid, &hash);
        Ok(Response::Redirect(url))
    } else {
        Err(Error::NotFound(format!("Can't find post {}", fpid)))
    }
}
