use std::collections::HashMap;

//use common::capi::get_browse;
use common::capi::get_submission_categories;
use common::capi::QueryLimit;
use common::capi::SubmissionCategory;
//use contentapi::*;

use common::*;
//use common::view::*;
use common::forms::*;
//use common::search::*;
use common::constants::*;
use common::response::*;
use common::render::layout::*;
use common::render::submissions::*;
use maud::*;

pub fn render(data: MainLayoutData, pages: Vec<capi::BrowseContent>, users: HashMap<i64, capi::User2>, search: PageSearch,
    categories: Vec<SubmissionCategory>) -> String 
{
    //Need to split category search into parts 
    layout(&data, html!{
        (data.links.style("/forpage/search.css"))
        (data.links.script("/forpage/search.js"))
        section {
            //Don't include an action so it just posts to the same url but with the form as params...?
            form."smallseparate compactform" method="GET" id="searchform" {
                div."smallseparate inline" {
                    label for="search-type" {"Type: "}
                    select #"search-type" name="subtype" {
                        @for (value,text) in SEARCHPAGETYPES {
                            option value=(value) selected[Some(*value) == search.subtype.as_deref()] { (text) }
                        }
                    }
                }
                //THIS needs to come from parameters! Don't know the categories available unless
                //we look at the database!
                div."smallseparate inline" for="search-category" 
                {
                    label for="search-category" {"Category:"}
                    select #"search-category" name="category" {
                        option value="0" { "Any" }
                        @for category in &categories {
                            //NOTE: this "selected" actually does work, it adds the attribute when appropriate. But
                            //the select is not showing this when the page loads, so it may be js or something
                            option data-for=(category.forcontent) value=(category.id) selected[Some(category.id) == search.category] { 
                                (category.name) 
                            }
                        }
                    }
                }
                @if search.subtype.as_deref() == Some(SBSPageType::PROGRAM) {
                    div."smallseparate inline" {
                        label for="search-system" {"System: "}
                        select #"search-system" name="system" {
                            @for (value,text) in SBSSYSTEMS {
                                option value=(value) selected[*value == search.system] { (text) }
                            }
                        }
                    }
                }
                dv."smallseparate inline" {
                    label for="search-order" {"Order: "}
                    select #"search-order" name="order" {
                        @for (value,text) in SEARCHPAGEORDERS {
                            option value=(value) selected[*value == search.order] { (text) }
                        }
                    }
                }
                div."smallseparate inline" {
                    label for="search-text" { "Search: " }
                    input."" #"search-text" type="text" name="search" value=[&search.search];
                }
                @if search.subtype.as_deref() == Some(SBSPageType::PROGRAM) {
                    div."smallseparate inline" {
                        label for="search-removed" { "Show removed: " }
                        input."" #"search-text" type="checkbox" name="removed" checked[search.removed] value="true";
                    }
                }
                div."smallseparate inline" {
                    label for="search-page" {"Page: "}
                    input."smallinput" #"search-page" type="text" name="page" value=(search.page); 
                }

                input type="submit" value="Update search";
            }
        }
        // All the pages (directly in the section?)
        section."results" {
            div."cardslist" {
                //Or maybe in here
                @for page in &pages {
                    (page_card2(&data.links, page, &users))
                }
            }
            //Generic pagelist generation (just need data)
            (page_navigation(&data, &search))
        }
    }).into_string()
}


// TODO: Make this generic across imagebrowse and here? Search has to impl some trait with get/set 
// page functions and clone, and .browsepagenav might need to go in base.css
fn page_navigation(data: &MainLayoutData, search: &PageSearch) -> Markup {
    let mut searchprev = search.clone();
    let mut searchnext = search.clone();
    searchprev.page = searchprev.page - 1;
    searchnext.page = searchnext.page + 1;
    html! {
        div."smallseparate browsepagenav" {
            @if let Ok(prevlink) = serde_urlencoded::to_string(searchprev) {
                a."coolbutton" href={(data.current())"?"(prevlink)} {"Previous"}
            }
            @if let Ok(nextlink) = serde_urlencoded::to_string(searchnext) {
                a."coolbutton" href={(data.current())"?"(nextlink)} {"Next"}
            }
        }
    }
}


pub async fn get_render(context: PageContext, search: PageSearch, per_page: i32) -> Result<Response, Error> 
{
    let pages = capi::get_browse(&context, &search, 
        QueryLimit { limit: Some(per_page), skip: Some(search.page * per_page)})?;
    let users = capi::get_users(&context, pages.iter().map(|x| x.create_user_id).collect())?;
    let users = view::map_users2(users);
    let categories = get_submission_categories(&context)?; //map_categories(categories);

    //Manually parse the search, because of the tag magic (no javascript)
    Ok(Response::Render(render(context.layout_data, pages,  users, search, categories)))
}
