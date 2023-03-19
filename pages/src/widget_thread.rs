use common::*;

use contentapi::*;
use contentapi::conversion::*;
use common::forms::*;
use common::forum::*;
use common::view::*;
use common::render::layout::*;
use common::render::forum::*;
use maud::*;


/// Rendering for the actual widget. The 
pub fn render(context: &mut PageContext, config: PostsConfig) -> String {
    let posts = render_posts(context, config);
    basic_skeleton(&context.layout_data, html! {
        title { "SmileBASIC Source Image Browser" }
        meta name="description" content="Simple image browser widget";
        (context.layout_data.links.style("/forpage/forum.css"))
        style { r#"
            body { 
                /* This shrinks the WHOLE page! */
                font-size: 0.85rem; 
                padding: var(--space_medium);
            }
            @media screen and (max-width: 30em) {
                font-size: 0.75rem; 
            }
        "# }
    }, html! {
        (posts)
    }).into_string()
}


pub async fn get_render(mut context: PageContext, query: ThreadQuery) -> Result<Response, Error> 
{
    if let Some(post_id) = query.reply 
    {
        //This is a WASTEFUL query for rendering this simple widget, at some point make this better!
        let pre_request = get_prepost_request(None, Some(post_id), None, None);

        //Go lookup all the 'initial' data, which everything except posts and users
        let pre_result = context.api_context.post_request_profiled_opt(&pre_request, "prepost").await?;

        //Pull out and parse all that stupid data. It's fun using strongly typed languages!! maybe...
        let mut categories_cleaned = CleanedPreCategory::from_many(cast_result_required::<Content>(&pre_result, CATEGORYKEY)?)?;
        let mut threads_raw = cast_result_required::<Content>(&pre_result, THREADKEY)?;

        //There must be one category, and one thread, otherwise return 404
        let thread = threads_raw.pop().ok_or(Error::NotFound(String::from("Could not find thread!")))?;
        let category = categories_cleaned.pop().ok_or(Error::NotFound(String::from("Could not find category!")))?;

        //OK NOW you can go lookup the posts, since we are sure about where in the postlist we want
        let after_request = get_reply_request(post_id);
        let after_result = context.api_context.post_request_profiled_opt(&after_request, "finishpost").await?;

        //Pull the data out of THAT request
        let messages_raw = cast_result_required::<Message>(&after_result, "message")?;
        let related_raw = cast_result_required::<Message>(&after_result, "related")?;
        let users_raw = cast_result_required::<User>(&after_result, "user")?;

        Ok(Response::Render(render(&mut context, PostsConfig::reply_mode(
            ForumThread::from_content(thread, &messages_raw, &category.stickies)?, 
            map_messages(related_raw),
            map_users(users_raw), 
            query.selected
        ))))
    }
    else {
        Err(Error::Other(String::from("No data provided; this widget requires at least 'reply'")))
    }
}