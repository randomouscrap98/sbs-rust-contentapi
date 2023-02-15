use common::forum::get_new_replydata;
use common::forum::get_replydata;
use contentapi::*;

use common::*;
use common::forms::*;
use common::render::*;
use common::render::forum::*;
use common::render::layout::*;
use contentapi::endpoints::ApiContext;
use maud::*;

//Rendering ALWAYS requires the form, even if it's just an empty one
pub fn render(data: MainLayoutData, form: PostForm, thread_info: Option<Content>, errors: Option<Vec<String>>) -> String 
{
    let mut title : Option<String> = None;
    let mut submit_value = "Submit post";

    //Assume it's new or not based on the values in the form. The form drives this render
    if form.id == 0 {
        if let Some(reply_id) = form.reply_id {
            title = Some(format!("Replying to post '{}'", reply_id));
            submit_value = "Post reply";
        }
        else if let Some(ref thread) = thread_info {
            title = Some(format!("New post in '{}'", opt_s!(thread.name)));
        }
    }
    else {
        title = Some(format!("Edit post: '{}'", form.id));
        submit_value = "Edit post";
    }

    layout(&data, html!{
        (data.links.style("/forpage/forum.css"))
        section {
            @if let Some(title) = title {
                h1 { (title) }
                //NOTE: NO ACTION! These kinds of pages always post to themselves
                form."editor" #"postedit_form" method="POST" {
                    (errorlist(errors))
                    input #"postedit_content_id" type="hidden" name="content_id" value=(form.content_id);
                    input #"postedit_id" type="hidden" name="id" value=(form.id);
                    @if let Some(reply_id) = form.reply_id {
                        input #"postedit_reply_id" type="hidden" name="reply_id" value=(reply_id);
                    }
                    label for="postedit_post" {"Post:"}
                    (post_textbox(Some("postedit_post"), Some("post"), None))
                    input type="submit" value=(submit_value);
                }
            }
            @else {
                h1."error" { "POST EDITOR CANNOT LOAD" }
            }
        }
    }).into_string()
}

//You can optimize this later I guess (if it really needs it...)
const THISCONTENTFIELDS : &str = "*";
const THISMESSAGEFIELDS : &str = "*";

pub async fn get_render(context: PageContext, thread_hash: Option<String>, post_id: Option<i64>, reply_id: Option<i64>) -> 
    Result<Response, Error> 
{
    let mut thread : Option<Content> = None;
    let mut form = PostForm::default();

    //This may get overwritten later if we have a pre-existing message
    form.reply_id = reply_id;

    if let Some(hash) = thread_hash {
        let c = context.api_context.get_content_by_hash(&hash, THISCONTENTFIELDS).await?;
        form.content_id = c.id.unwrap(); 
        thread = Some(c);
    }
    if let Some(post_id) = post_id {
        let post = context.api_context.get_message_by_id(post_id, THISMESSAGEFIELDS).await?;
        form.content_id = post.contentId.unwrap();
        form.id = post.id.unwrap();
        if let Some(reply_data) = get_replydata(&post) {
            form.reply_id = Some(reply_data.direct);
        }
    }

    Ok(Response::Render(render(context.layout_data, form, thread, None)))
}

/// Craft the message to be written to the api for the given post form
pub async fn construct_post_message(context: &ApiContext, form: &PostForm) 
    -> Result<Message, Error>
{
    let mut message;
    
    if form.id > 0 {
        //Use all the values from the original message. You can't "move" messages fyi...
        message = context.get_message_by_id(form.id, THISMESSAGEFIELDS).await?;
    }
    else {
        message = Message::default();
        message.contentId = Some(form.content_id);
        let mut values = make_values! {
            "markup": "bbcode"
        };
        //If we have a reply, add the appropriate reply-replated values
        if let Some(reply_id) = form.reply_id {
            let reply_to = context.get_message_by_id(reply_id, THISMESSAGEFIELDS).await?;
            let reply_data = get_new_replydata(&reply_to);
            reply_data.write_to_values(&mut values);
        }
        message.values = Some(values);
    }
    message.text = Some(form.post.clone()); 

    Ok(message)
}

pub async fn post_render(context: PageContext, form: PostForm) ->
    Result<Response, Error>
{
    if let Some(ref _user) = context.layout_data.user 
    {
        //This one, we throw all the way, since we can't re-render the page without the parent anyway
        let thread = context.api_context.get_content_by_id(form.content_id, THISCONTENTFIELDS).await?;
        let mut written_post : Option<Message> = None;
        let mut errors = Vec::new();

        match construct_post_message(&context.api_context, &form).await {
            Ok(message) =>
            {
                match context.api_context.post_message(&message).await { 
                    Ok(posted_post) => {
                        written_post = Some(posted_post);
                    },
                    Err(e) => { errors.push(e.to_user_string()); }
                }
            },
            Err(e) => { errors.push(e.to_user_string()); }
        }

        if errors.is_empty() {
            //If there are no errors, we go to the new page
            Ok(Response::Redirect(
                if let Some(ref post) = written_post {
                    context.layout_data.links.forum_post(post, &thread)
                }
                else {
                    Err(Error::Other(String::from("Some internal error occurred, preventing the new thread from being shown! No errors produced, but no thread data found!")))?
                })
            )
        }
        else {
            //Otherwise, we stay here and show all the terrifying errors
            Ok(Response::Render(render(context.layout_data, form, Some(thread), Some(errors))))
        }
    }
    else {
        Err(Error::Other(String::from("Not logged in!")))
    }
}

pub async fn delete_render(context: PageContext, post_id: i64) ->
    Result<Response, Error>
{
    //This is a VERY DUMB delete endpoint, because it just passes it through to the backend. Then the backend will
    //produce errors and the resulting screen will just be one of the ugly 400 screens (which we might spruce up).
    //I don't care much about it because the delete thread isn't a form, its just a button, so putting the usual
    //error box doesn't really work.
    context.api_context.post_delete_message(post_id).await?;

    //Again, super dumb
    Ok(Response::Redirect(context.layout_data.links.activity()))
}
