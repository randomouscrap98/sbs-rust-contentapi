use common::{*, render::submissions::pageicon_limited};
use common::constants::{THREADTYPES};
use common::render::layout::*;
use contentapi::*;
use contentapi::conversion::cast_result_required;
use contentapi::forms::QueryImage;
use maud::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct SearchAllForm {
    pub search : Option<String>
}

pub enum SearchAllResult {
    User(contentapi::User),
    Content(contentapi::Content)
}

//This will render the entire index! It's a handler WITH the template in it! Maybe that's kinda weird? who knows...
//pub fn index(data: MainLayoutData) -> Result<impl warp::Reply, Infallible>{
pub fn render(data: MainLayoutData, search_results: Option<Vec<SearchAllResult>>, search_form: SearchAllForm) -> String {
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
                }
            }
            @if let Some(results) = search_results {
                div #"searchresults" {
                    @if results.len() > 0 {
                        @for ref result in results {
                            div."smallseparate searchitem" {
                                @match result {
                                    SearchAllResult::Content(content) => {
                                        span."threadicon searchicon"  { (pageicon_limited(&data.links, content, 1)) }
                                        a."pagetitle flatlink searchname" target="_top" href=(data.links.forum_thread(content)) { (opt_s!(content.name)) }
                                    },
                                    SearchAllResult::User(user) => {
                                        span."searchicon" { img."avatar" src=(data.links.image(&user.avatar, &QueryImage { crop: Some(true), size: Some(100) })); }
                                        a."username flatlink searchname" target="_top" href=(data.links.user(user)) { (user.username) }
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

pub async fn get_render(mut context: PageContext, search_form: SearchAllForm) -> Result<Response, Error> 
{
    let mut result : Option<Vec<SearchAllResult>> = None;
    //Don't do a search unless a search was given of course
    if let Some(ref search) = search_form.search {
        if search.len() > 0 {
            let mut request = FullRequest::new();
            add_value!(request, "allowed_types", THREADTYPES);

            if search.len() < 2 {
                //If the search is too short, do the significantly faster search of 'starts with'
                add_value!(request, "search", format!("{}%", search));
            }
            else {
                //Otherwise, we can do the slow search of 'contains'
                add_value!(request, "search", format!("%{}%", search));
            }

            //Search for content within the allowed forum types which contain the search, whether name or keywords
            let content_request = build_request!(
                RequestType::content,
                String::from("id,name,literalType,hash"),
                format!("literalType in @allowed_types and name like @search or !keywordlike(@search)")
            );
            request.requests.push(content_request);

            //And search for user
            let user_request = build_request!(
                RequestType::user,
                String::from("*"), //User structure has lots of required fields
                format!("username like @search")
            );
            request.requests.push(user_request);

            let search_result = context.api_context.post_request_profiled_opt(&request, "searchall").await?;
            let content = cast_result_required::<Content>(&search_result, "content")?;
            let users = cast_result_required::<User>(&search_result, "user")?;

            let mut result_combined : Vec<SearchAllResult> = content.into_iter().map(|x| SearchAllResult::Content(x)).collect();
            result_combined.extend(users.into_iter().map(|x| SearchAllResult::User(x)));

            let searchlower = search.to_ascii_lowercase();
            result_combined.sort_by(|a, b| search_score(b, &searchlower).partial_cmp(&search_score(a, &searchlower)).unwrap());

            result = Some(result_combined);
        }
    }
    Ok(Response::Render(render(context.layout_data, result, search_form)))
}

/// Score the item based on the given PRE-LOWERCASED search term
fn search_score(item: &SearchAllResult, search: &str) -> f32 {
    match item {
        SearchAllResult::Content(c) => {
            let mut score = 0.0;
            if let Some(ref name) = c.name {
                score += search_score_name(name, search);
            }
            if let Some(ref keywords) = c.keywords {
                for k in keywords {
                    score += 0.5 * search_score_name(k, search);
                }
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
fn search_score_name(name: &str, search: &str) -> f32
{
    let mut score = 0.0;
    let namelower = name.to_ascii_lowercase();
    if namelower.starts_with(search) { score += 1.0; }
    if namelower.contains(search) { score += 1.0; }
    score
}