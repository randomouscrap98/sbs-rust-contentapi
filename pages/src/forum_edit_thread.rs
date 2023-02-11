//use std::collections::HashMap;

use contentapi::conversion::*;
use contentapi::*;
//use contentapi::endpoints::ApiContext;


use common::*;
//use common::constants::*;
use common::forum::*;
use common::render::*;
use common::render::forum::*;
use common::render::layout::*;
//use common::pagination::*;
use maud::*;


pub fn render(data: MainLayoutData, new_in_category: Option<Content>, edit_thread: Option<Content>, errors: Option<Vec<String>>) -> String 
{
    let mut title : Option<String> = None;
    let mut thread_name : Option<&str> = None;

    if let Some(ref category) = new_in_category {
        title = Some(format!("New thread in '{}'", opt_s!(category.name)))
    }
    else if let Some(ref thread) = edit_thread {
        title = Some(format!("Edit thread: '{}'", opt_s!(thread.name)));
        thread_name = thread.name.as_deref();
    }

    layout(&data, html!{
        (data.links.style("/forpage/forum.css"))
        section {
            @if let Some(title) = title {
                h1 { (title) }
                //NOTE: NO ACTION! These kinds of pages always post to themselves
                form."editor" #"threadeditform" method="POST" {
                    (errorlist(errors))
                    @if let Some(ref category) = new_in_category {
                        input #"threadedit_category" type="hidden" name="category" value=(opt_s!(category.hash));
                    }
                    label for="threadedit_title"{"Thread title:"}
                    input #"threadedit_title" type="text" name="title" value=(opt_s!(thread_name));
                    @if edit_thread.is_none() {
                        label for="threadedit_post" {"Post:"}
                        (post_textbox(Some("threadedit_post"), Some("post"), None))
                        input type="submit" value="Post thread";
                    }
                    @else {
                        input type="submit" value="Update thread";
                    }
                }
            }
            @else {
                h1."error" { "THREAD EDITOR CANNOT LOAD" }
            }
        }
    }).into_string()
}

pub async fn get_render(mut context: PageContext, category_hash: Option<String>, thread_hash: Option<String>) -> 
    Result<Response, Error> 
{
    let mut category : Option<Content> = None;
    let mut thread : Option<Content> = None;

    if category_hash.is_some() {
        let request = get_category_request(category_hash, None);
        let category_result = context.api_context.post_request_profiled_opt(&request, "getcategory").await?;
        let mut categories = cast_result_required::<Content>(&category_result, CATEGORYKEY)?;
        category = categories.pop();
    }
    if thread_hash.is_some() {
        let request = get_prepost_request(None, None, None, thread_hash); //get_category_request(category_hash, None);
        let thread_result = context.api_context.post_request_profiled_opt(&request, "getthread").await?;
        let mut threads = cast_result_required::<Content>(&thread_result, THREADKEY)?;
        thread = threads.pop();
    }

    Ok(Response::Render(render(context.layout_data, category, thread, None)))
}
