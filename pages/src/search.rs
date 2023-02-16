use std::collections::HashMap;

use contentapi::{*, conversion::map_users};

use common::*;
use common::forms::*;
use common::submissions::*;
use common::constants::*;
use common::render::layout::*;
use common::render::submissions::*;
use maud::*;

pub fn render(data: MainLayoutData, pages: Vec<Content>, users: HashMap<i64, User>, search: PageSearch,
    categories: Vec<Category>) -> String 
{
    //Need to split category search into parts 
    //let search_system = match &search.system { Some(system) => system, None => };
    layout(&data, html!{
        (data.links.style("/forpage/search.css"))
        (data.links.script("/forpage/search.js"))
        section {
            //Don't include an action so it just posts to the same url but with the form as params...?
            form."smallseparate" method="GET" id="searchform" {
                label."inline" for="search-type" {
                    span{"Type: "}
                    select #"search-type" name="subtype" {
                        @for (value,text) in SEARCHPAGETYPES {
                            option value=(value) selected[Some(*value) == search.subtype.as_deref()] { (text) }
                        }
                    }
                }
                //THIS needs to come from parameters! Don't know the categories available unless
                //we look at the database!
                label."inline" for="search-category" 
                {
                    span{"Category:"}
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
                    label."inline" for="search-system" {
                        span{"System: "}
                        select #"search-system" name="system" {
                            @for (value,text) in SBSSYSTEMS {
                                option value=(value) selected[*value == search.system] { (text) }
                            }
                        }
                    }
                }
                label."inline" for="search-order" {
                    span{"Order: "}
                    select #"search-order" name="order" {
                        @for (value,text) in SEARCHPAGEORDERS {
                            option value=(value) selected[*value == search.order] { (text) }
                        }
                    }
                }
                label."inline" for="search-text" {
                    span { "Search: " }
                    input."" #"search-text" type="text" name="search" value=[&search.search];
                }
                @if search.subtype.as_deref() == Some(SBSPageType::PROGRAM) {
                    label."inline" for="search-removed" {
                        span { "Show removed: " }
                        input."" #"search-text" type="checkbox" name="removed" checked[search.removed] value="true";
                    }
                }
                label."inline" for="search-page" {
                    span {"Page: "}
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
                    (page_card(&data.links, page, &users))
                }
            }
            //Generic pagelist generation (just need data)
            (page_navigation(&data, &search))
            @if let Some(ref _user) = data.user {
                div."pagelist smallseparate" {
                    a."coolbutton" #"newprogram" href=(data.links.page_editor_new(SBSPageType::PROGRAM)) { "New Program" }
                    a."coolbutton" #"newresource" href=(data.links.page_editor_new(SBSPageType::RESOURCE)) { "New Resource" }
                }
            }
        }
        //@if let Some(ref _user) = data.user {
        //    section #"newpagecontrols" {
        //        div."pagelist smallseparate" {
        //            a."coolbutton" #"newprogram" href=(data.links.page_editor_new(SBSPageType::PROGRAM)) { "New Program" }
        //            a."coolbutton" #"newresource" href=(data.links.page_editor_new(SBSPageType::RESOURCE)) { "New Resource" }
        //        }
        //    }
        //}
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


pub struct Category {
    pub id: i64,
    pub name: String,
    pub forcontent: String
}

pub async fn get_render(context: PageContext, search: PageSearch, per_page: i32) -> Result<Response, Error> 
{
    let request = get_search_request(&search, per_page);

    let result = context.api_context.post_request(&request).await?;
    //println!("RESULT: {:#?}", &result);
    let pages = conversion::cast_result_safe::<Content>(&result, "content")?;
    let users = conversion::cast_result_safe::<User>(&result, "user")?;
    let categories = conversion::cast_result_safe::<Content>(&result, "categories")?;
    let users = map_users(users);

    let categories = categories.into_iter().map(|c| {
        Category {
            id: c.id.unwrap_or(0),
            name: c.name.unwrap_or_else(|| String::from("")), //Only evaluated on failure
            forcontent: c.values
                .and_then(|v| v.get(SBSValue::FORCONTENT).and_then(|v2| v2.as_str()).and_then(|v3| Some(String::from(v3))))
                .unwrap_or_else(|| String::from(""))
        }
    }).collect::<Vec<Category>>();

    //Manually parse the search, because of the tag magic (no javascript)
    //Err(Error::Other(String::from("wow")))
    Ok(Response::Render(render(context.layout_data, pages,  users, search, categories)))
}