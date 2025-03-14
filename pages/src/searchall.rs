use common::capi::*;
use common::render::layout::*;
use common::response::*;
use common::{render::submissions::pageicon2, *};
use maud::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct SearchAllForm {
    pub search: Option<String>,
}

//This will render the entire index! It's a handler WITH the template in it! Maybe that's kinda weird? who knows...
pub fn render(
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
                                        span."searchicon" { img."avatar" src=(data.links.image(&user.avatar, contentapi::QueryImage::Cropped100)); }
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
            let mut result_raw = capi::get_searchall(&context, search)?;
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
