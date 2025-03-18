use super::common::contentapi::*;
use super::common::render::*;
use super::context::*;
use crate::layout::*;
use crate::links::*;
use crate::opt_s;
use crate::response::*;

//use common::{render::submissions::pageicon2, *};
use maud::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct SearchAllForm {
    pub search: Option<String>,
}

#[derive(Clone, Debug)]
struct SearchAllUser {
    //pub id: i64,
    pub username: String,
    pub avatar: String,
}

#[derive(Clone, Debug)]
struct SearchAllContent {
    //pub id: i64,
    pub hash: String,
    pub name: String,
    pub literal_type: String,
    pub values: HashMap<String, String>,
    pub keywords: Vec<String>,
}

enum SearchAllResult {
    User(SearchAllUser),
    Content(SearchAllContent),
}

fn get_searchall(ctx: &PageContext, search: &str) -> Result<Vec<SearchAllResult>, Error> {
    let mut result: Vec<SearchAllResult> = Vec::new();
    let rsearch = if search.len() < 2 {
        format!("{}%", search) // Short search length means more optimized "begins with"
    } else {
        format!("%{}%", search)
    };

    // First, search users
    let query = format!(
        "SELECT id,username,avatar FROM users WHERE {} AND username LIKE ?",
        COMMONUSER
    );
    let mut stmt = ctx.dbcon.prepare(&query)?;
    result.extend(easy_query(
        (&mut stmt, &query),
        rusqlite::params![&rsearch],
        |row| {
            Ok(SearchAllResult::User(SearchAllUser {
                //id: row.get(0)?,
                username: row.get(1)?,
                avatar: row.get(2)?,
            }))
        },
    )?);

    // And then, content. It's a bit silly, but for I think slightly better performance,
    // we query the set of all content that matches the keywords separately from querying
    // the actual keyword list. There are better ways to do this, but I'm lazy, sorry
    // future self?
    let query = format!(
        "SELECT id,hash,name,COALESCE(literalType, '') FROM content WHERE {} AND literalType IN ({}) AND (name LIKE ? OR id IN (SELECT contentId FROM content_keywords WHERE `value` LIKE ?))",
        COMMONCONTENT,
        params_list(THREADTYPES.len())
    );

    let mut params: Vec<&dyn rusqlite::types::ToSql> = Vec::new();
    for tt in THREADTYPES.iter() {
        params.push(tt);
    }
    params.push(&rsearch);
    params.push(&rsearch);

    let mut stmt = ctx.dbcon.prepare(&query)?;
    let mut vstmt = ctx.dbcon.prepare(VALUESELECT)?;
    let mut kstmt = ctx.dbcon.prepare(KEYWORDSELECT)?;
    result.extend(easy_query((&mut stmt, &query), params.as_slice(), |row| {
        let id: i64 = row.get(0)?;
        Ok(SearchAllResult::Content(SearchAllContent {
            //id,
            hash: row.get(1)?,
            name: row.get(2)?,
            literal_type: row.get(3)?,
            values: gather_values(&mut vstmt, id)?,
            keywords: gather_keywords(&mut kstmt, id)?,
        }))
    })?);

    Ok(result)
}

//This will render the entire index! It's a handler WITH the template in it! Maybe that's kinda weird? who knows...
fn render(
    data: MainLayoutData,
    search_results: Option<Vec<SearchAllResult>>,
    search_form: SearchAllForm,
) -> String {
    layout(&data, html!{
        (data.links.style("/forpage/forum.css"))
        (data.links.style("/forpage/searchall.css"))
        section {
            //Want this to be enter to submit, don't remember if that's the default
            form #"searchallform" method="GET" {
                h3 #"searchtitle" { "Search website content" }
                div #"searchline" {
                    input #"searchinput" type="text" name="search" value=(opt_s!(search_form.search)) placeholder="Search";
                    input type="submit" value="🔎";
                    @if let Some(ref results) = search_results {
                        div."aside" #"resultcount" { (results.len()) " result(s)" }
                    }
                }
            }
            @if let Some(results) = search_results {
                div #"searchresults" {
                    @if results.len() > 0 {
                        @for ref result in results {
                            div."smallseparate resultitem" {
                                @match result {
                                    SearchAllResult::Content(content) => {
                                        span."threadicon searchicon"  { (pageicon2(&data.links, &content.values, &content.literal_type, 1)) }
                                        a."pagetitle flatlink searchname" target="_top" href=(data.links.forum_thread_unsafe(&content.hash)) { (content.name) }
                                    },
                                    SearchAllResult::User(user) => {
                                        span."searchicon" { img."avatar" src=(data.links.image(&user.avatar, QueryImage::Cropped100)); }
                                        a."username flatlink searchname" target="_top" href=(data.links.user_unsafe(&user.username)) { (user.username) }
                                    }
                                }
                            }
                        }
                    }
                    @else {
                        div."aside" #"noresultstext" { "No results!" }
                    }
                }
            }
        }
    }).into_string()
}

//There is no post, searching is done in the GET params

pub async fn get_render(
    context: PageContext,
    search_form: SearchAllForm,
) -> Result<Response, Error> {
    let mut result: Option<Vec<SearchAllResult>> = None;
    //Don't do a search unless a search was given of course
    if let Some(ref search) = search_form.search {
        if search.len() > 0 {
            let searchlower = search.to_ascii_lowercase();
            let mut result_raw = get_searchall(&context, search)?;
            result_raw.sort_by(|a, b| {
                search_score(b, &searchlower)
                    .partial_cmp(&search_score(a, &searchlower))
                    .unwrap()
            });
            // for rr in result_raw.iter() {
            //     if let SearchAllResult::Content(c) = rr {
            //         println!("Keywords: {:?}", c.keywords);
            //     }
            // }

            result = Some(result_raw);
        }
    }
    Ok(Response::Render(render(
        context.layout_data,
        result,
        search_form,
    )))
}

/// Score the item based on the given PRE-LOWERCASED search term
fn search_score(item: &SearchAllResult, search: &str) -> f32 {
    match item {
        SearchAllResult::Content(c) => {
            let mut score = 0.0;
            score += search_score_name(&c.name, search);
            for k in &c.keywords {
                score += 0.5 * search_score_name(k, search);
            }
            score
        }
        SearchAllResult::User(u) => {
            let mut score = 0.0;
            score += search_score_name(&u.username, search);
            score
        }
    }
}

/// Score the name based on the given PRE-LOWERCASED search term
fn search_score_name(name: &str, search: &str) -> f32 {
    let mut score = 0.0;
    let namelower = name.to_ascii_lowercase();
    if namelower.starts_with(search) {
        score += 1.0;
    }
    if namelower.contains(search) {
        score += 1.0;
    }
    score
}
