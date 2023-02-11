//use std::collections::HashMap;

use contentapi::conversion::*;
use contentapi::*;
use contentapi::endpoints::ApiContext;


use common::*;
use common::constants::*;
use common::forum::*;
use common::render::*;
use common::render::forum::*;
use common::render::layout::*;
use common::pagination::*;
use maud::*;


pub fn render(data: MainLayoutData, new_in_category: Option<Content>, edit_thread: Option<Content>, errors: Option<Vec<String>>) -> String 
{
    let title : Option<String> = None;

    if let Some(category) = new_in_category {
        title = Some(format!("New thread in '{}'", opt_s!(category.name)));
    }
    if let Some(thread) = edit_thread {
        title = Some(format!("Edit thread: '{}'", opt_s!(thread.name)));
    }

    layout(&data, html!{
        (data.links.style("/forpage/forum.css"))
        section {
            @if let Some(title) = title {
                h1 { (opt_s!(title)) }
                form."editor" #"threadeditform" method="POST" action=(data.current()) {
                    (errorlist(errors))
                    @if let Some(category) = new_in_category {
                        input #"threadedit_category" type="hidden" name="category" value=(opt_s!(category.hash));
                    }
                    label for="threadedit_title"{"Thread title:"}
                    input #"threadedit_title" type="text" name="title" value=[edit_thread.and_then(|t| t.name)];
                    @if edit_thread.is_none() {
                        label for="threadedit_post" {"Post:"}
                        (post_textbox(Some("threadedit_post"), Some("post"), None))
                    }
                }
            }
            @else {
                h1."error" { "THREAD EDITOR CANNOT LOAD" }
            }
        }
    }).into_string()
}

pub async fn get_render(context: PageContext, category_hash: Option<String>, thread_hash: Option<String>) -> 
    Result<Response, Error> 
{
}
