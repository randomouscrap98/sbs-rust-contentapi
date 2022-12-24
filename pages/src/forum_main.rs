
use contentapi::*;
use contentapi::conversion::*;
use contentapi::endpoints::ApiContext;

use common::*;
use common::render::*;
use common::render::layout::*;
use common::forum::*;
use maud::*;

pub fn render(data: MainLayoutData, categories: Vec<ForumCategory>) -> String {
    layout(&data, html!{
        (data.links.style("/forpage/forum.css"))
        section { h1 { "Forum Topics" } }
        section {
            @for (index, category_container) in categories.iter().enumerate() {
                @let category = &category_container.category;
                div."category" {
                    div."categoryinfo" {
                        h1 { a."flatlink" title={"ID: "(i(&category.id))} href=(data.links.forum_category(&category)) {(opt_s!(category.name))} }
                        p."aside" {(opt_s!(category.description))}
                    }
                    div."foruminfo aside mediumseparate" {
                        div { b{"Threads: "} (category_container.threads_count) }
                        @if let Some(thread) = category_container.threads.get(0) {
                            div {
                                @if let Some(post) = thread.posts.get(0) {
                                    b { time datetime=(d(&post.createDate)) { (timeago_o(&post.createDate)) } }
                                    ": "
                                    a."flatlink" title={"ID: "(i(&thread.thread.id))} href=(data.links.forum_post(post, &thread.thread)) { (opt_s!(thread.thread.name)) } 
                                }
                            }
                        }
                    }
                }
                @if index < categories.len() - 1 {
                    hr;
                }
            }
        }
    }).into_string()
}

async fn build_categories_with_threads(mut context: ApiContext, categories_cleaned: Vec<CleanedPreCategory>, limit: i32, skip: i32) -> 
    Result<Vec<ForumCategory>, Error> 
{
    //Next request: get the complicated dataset for each category (this somehow includes comments???)
    let thread_request = get_thread_request(&categories_cleaned, limit, skip, false); 
    let thread_result = context.post_request_profiled_opt(&thread_request, "threads").await?;

    let messages_raw = cast_result_required::<Message>(&thread_result, "message")?;

    let mut categories = Vec::new();

    for category in categories_cleaned {
        categories.push(ForumCategory::from_result(category, &thread_result, &messages_raw)?);
    }

    Ok(categories)
}


pub async fn get_render(mut context: PageContext, order: &Vec<String>, show_threads: i32) -> Result<Response, Error> 
{
    //First request: just get categories
    let request = get_category_request(None, None);
    let category_result = context.api_context.post_request_profiled_opt(&request, "categories").await?;
    let mut categories_cleaned = CleanedPreCategory::from_many(cast_result_required::<Content>(&category_result, CATEGORYKEY)?)?;

    //Sort the categories by their name AGAINST the default list in the config. So, it should sort the categories
    //by the order defined in the config, with stuff not present going at the end. Tiebreakers are resolved alphabetically
    categories_cleaned.sort_by_key(|category| {
        //Nicole made this a tuple so tiebreakers are sorted alphabetically, which is coool
        (order.iter().position(
            |prefix| category.name.starts_with(prefix)).unwrap_or(usize::MAX), category.name.clone())
    });

    let categories = build_categories_with_threads(context.api_context, categories_cleaned, show_threads, 0).await?;

    Ok(Response::Render(render(context.layout_data, categories)))
}