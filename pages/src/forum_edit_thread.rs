//use std::collections::HashMap;

use common::constants::SBSPageType;
//use contentapi::conversion::*;
use contentapi::*;
//use contentapi::endpoints::ApiContext;


use common::*;
//use common::constants::*;
use common::forms::*;
//use common::forum::*;
use common::render::*;
use common::render::forum::*;
use common::render::layout::*;
//use common::pagination::*;
use maud::*;

//Rendering ALWAYS requires the form, even if it's just an empty one
pub fn render(data: MainLayoutData, form: ThreadForm, category_info: Option<Content>, errors: Option<Vec<String>>) -> String 
{
    let mut title : Option<String> = None;
    let mut edit = false;

    //Assume it's new or not based on the values in the form. The form drives this render
    if form.id == 0 {
        if let Some(ref category) = category_info {
            title = Some(format!("New thread in '{}'", opt_s!(category.name)));
        }
    }
    else {
        title = Some(format!("Edit thread: '{}'", form.title));
        edit = true;
    }

    layout(&data, html!{
        (data.links.style("/forpage/forum.css"))
        section {
            @if let Some(title) = title {
                h1 { (title) }
                //NOTE: NO ACTION! These kinds of pages always post to themselves
                form."editor" #"threadedit_form" method="POST" {
                    (errorlist(errors))
                    input #"threadedit_parent_id" type="hidden" name="parent_id" value=(form.parent_id);
                    label for="threadedit_title"{"Thread title:"}
                    input #"threadedit_title" type="text" name="title" value=(form.title) required;
                    input #"threadedit_id" type="hidden" name="id" value=(form.id);
                    @if edit {
                        input type="submit" value="Update thread";
                    }
                    @else {
                        label for="threadedit_post" {"Post:"}
                        (post_textbox(Some("threadedit_post"), Some("post"), None))
                        input type="submit" value="Post thread";
                    }
                }
            }
            @else {
                h1."error" { "THREAD EDITOR CANNOT LOAD" }
            }
        }
    }).into_string()
}

//You can optimize this later I guess (if it really needs it...)
const THISCONTENTFIELDS : &str = "*";

pub async fn get_render(context: PageContext, category_hash: Option<String>, thread_hash: Option<String>) -> 
    Result<Response, Error> 
{
    let mut category : Option<Content> = None;
    let mut form = ThreadForm::default();

    if let Some(hash) = category_hash {
        let c = context.api_context.get_content_by_hash(&hash, THISCONTENTFIELDS).await?;
        form.parent_id = c.id.unwrap(); 
        category = Some(c);
    }
    if let Some(hash) = thread_hash {
        let thread = context.api_context.get_content_by_hash(&hash, THISCONTENTFIELDS).await?;
        form.title = thread.name.unwrap(); 
        form.parent_id = thread.parentId.unwrap(); 
        form.id = thread.id.unwrap(); 
    }

    Ok(Response::Render(render(context.layout_data, form, category, None)))
}

pub async fn post_render(context: PageContext, form: ThreadForm) ->
    Result<Response, Error>
{
    //Creating a thread will show up as two events, and requires two inserts. How do we approach this?
    //The history will show "this user created thread 'whatever'" and then immediately posting. Actually,
    //go see what this looks like in person. It looks fine, just leave it for now.

    //So, we use the api to create content, then on success we add our post with defaults.
    if let Some(ref _user) = context.layout_data.user 
    {
        //This one, we throw all the way, since we can't re-render the page without the parent anyway
        let category = context.api_context.get_content_by_id(form.parent_id, THISCONTENTFIELDS).await?;
        let mut thread : Option<Content> = None;
        let mut post : Option<Message> = None;

        let mut errors = Vec::new();
        let mut content = Content::default();
        //note: the hash it autogenerated from the name (hopefully)
        content.text = Some(String::from("")); //Threads have no text... kinda weird but just easier
        content.id = Some(form.id);
        content.name = Some(form.title.clone());
        content.parentId = Some(form.parent_id);
        content.contentType = Some(ContentType::PAGE);
        content.literalType = Some(SBSPageType::FORUMTHREAD.to_string());
        content.permissions = Some(make_permissions! {
            "0": "CR" //Create so people can post on your "wall" (idk if that'll ever happen)
        });
        content.values = Some(make_values! {
            "markup": "bbcode"
        });
        match context.api_context.post_content(&content).await { //.map_err(|e| e.into())
            Ok(post_thread) =>
            {
                let thread_id = post_thread.id;
                thread = Some(post_thread);

                if let Some(ref text) = form.post {
                    //If a post field was given, also create a new post in the thread. We don't have
                    //proper error handling for if the thread succeeds but the post does not.
                    let mut message = Message::default();
                    message.text = Some(text.clone());
                    message.contentId = thread_id;
                    message.values = Some(make_values! {
                        "markup": "bbcode"
                    });
                    match context.api_context.post_message(&message).await {
                        Ok(written_post) =>
                        {
                            post = Some(written_post);
                        },
                        Err(e) => { errors.push(e.to_user_string()); }
                    }
                }
            },
            Err(e) => { errors.push(e.to_user_string()); }
        }

        if errors.is_empty() {
            //If there are no errors, we go to the new page
            Ok(Response::Redirect(
                if let Some(ref thread) = thread {
                    if let Some(ref post) = post { context.layout_data.links.forum_post(post, thread) } 
                    else { context.layout_data.links.forum_thread(thread) }
                }
                else {
                    Err(Error::Other(String::from("Some internal error occurred, preventing the new thread from being shown! No errors produced, but no thread data found!")))?
                })
            )
        }
        else {
            //Otherwise, we stay here and show all the terrifying errors
            Ok(Response::Render(render(context.layout_data, form, Some(category), Some(errors))))
        }
    }
    else {
        Err(Error::Other(String::from("Not logged in!")))
    }
}

//No editing for now
//pub async fn post_edit_render(mut context: PageContext, id: i64, parent_id: i64, title: String)
//{
//
//}