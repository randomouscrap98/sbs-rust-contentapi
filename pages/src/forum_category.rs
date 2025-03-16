// use contentapi::conversion::*;
// use contentapi::endpoints::ApiContext;
// use contentapi::*;

use common::capi::*;
use common::constants::*;
// use common::forum::*;
use common::pagination::*;
use common::render::forum::*;
use common::render::layout::*;
use common::render::*;
use common::response::*;
use common::*;
use maud::*;

pub fn render(
    mut data: MainLayoutData,
    category: ForumCategory2,
    threads: Vec<ForumThread2>,
    path: Vec<ForumPathItem>,
    pages: Vec<PagelistItem>,
) -> String {
    if category.literal_type == SBSPageType::SUBMISSIONS {
        data.override_nav_path = Some("/search");
    } else if category.literal_type == SBSPageType::DIRECTMESSAGES {
        data.override_nav_path = Some("/userhome");
    }

    layout(&data, html!{
        (data.links.style("/forpage/forum.css"))
        section {
            h1 { (category.name) }
            p."aside" {(category.description)}
            (forum_path(&data.links, &path))
        }
        section {
            //Only care about 'unless' in the main list, the only time this DOES work is if there are ONLY stickies
            @for (index,thread) in threads.iter().enumerate() {
                (thread_item(&data.links, thread))
                @if index < threads.len() - 1 {
                    hr."smaller";
                }
            }
            div."smallseparate pagelist" {
                @for page in pages {
                    a."current"[page.current] href={(data.links.forum_category_unsafe(&category.hash))"?page="(page.page)} { (page.text) }
                }
            }
        }
    }).into_string()
}

pub fn thread_item(links: &LinkConfig, thread: &ForumThread2) -> Markup {
    html! {
        div."thread" {
            div."threadinfo" {
                h3 { a."flatlink" href=(links.forum_thread_unsafe(&thread.hash)) { (thread.name) } }
            }
            div."foruminfo aside mediumseparate" {
                (threadicon2(links, thread))
                div { b { "Posts: " } (thread.posts_count) }
                div {
                    b { "Created: " }
                    time datetime=(dd(&thread.create_date)) { (timeago(&thread.create_date)) }
                }
                @if thread.max_post_id > 0 {
                    div {
                        b { "Last: " }
                        a."flatlink" href=(links.forum_post_unsafe(thread.max_post_id, &thread.hash)) {
                            (thread.max_post_id)
                            //time datetime=(d(&post.createDate)) { (timeago_o(&post.createDate)) }
                        }
                    }
                }
            }
        }
    }
}

// async fn build_categories_with_threads(
//     context: &mut ApiContext,
//     categories_cleaned: Vec<CleanedPreCategory>,
//     limit: i32,
//     skip: i32,
// ) -> Result<Vec<ForumCategory>, Error> {
//     //Next request: get the complicated dataset for each category (this somehow includes comments???)
//     let thread_request = get_thread_request(&categories_cleaned, limit, skip);
//     let thread_result = context.post_request(&thread_request).await?;
//
//     let messages_raw = cast_result_required::<Message>(&thread_result, "message")?;
//
//     let mut categories = Vec::new();
//
//     for category in categories_cleaned {
//         categories.push(ForumCategory::from_result(
//             category,
//             &thread_result,
//             &messages_raw,
//         )?);
//     }
//
//     Ok(categories)
// }

async fn render_threads(
    context: PageContext,
    idhash: Option<OldIdOrHash>,
    per_page: i32,
    page: Option<i32>,
) -> Result<Response, Error> {
    let category = get_forum_categories(&context, idhash.clone())?
        .pop()
        .ok_or(Error::NotFound(String::from(
            "Couldn't find that category (direct)",
        )))?;

    let page = page.unwrap_or(1) - 1;

    let threads = get_threads(
        &context,
        None,
        Some(category.id),
        QueryLimit {
            limit: Some(per_page),
            skip: Some(per_page * page),
        },
    )?;

    //let category_result = context.api_context.post_request(&category_request).await?;
    // let categories_cleaned = CleanedPreCategory::from_many(cast_result_required::<Content>(
    //     &category_result,
    //     CATEGORYKEY,
    // )?)?;
    // let mut categories = build_categories_with_threads(
    //     &mut context.api_context,
    //     categories_cleaned,
    //     per_page,
    //     page * per_page,
    // )
    // .await?;

    // //TODO: Might want to add data to these RouteErrors?
    // let category = categories
    //     .pop()
    //     .ok_or(Error::NotFound(String::from("Couldn't find that category")))?;
    let pagelist = get_pagelist(category.threads_count, per_page, page);

    //let mut real_category = get_categories(&context, Some(category.id))?;

    let path = vec![
        ForumPathItem::root(),
        ForumPathItem::from_category2(&category),
    ];
    Ok(Response::Render(render(
        context.layout_data,
        category,
        threads,
        path,
        pagelist,
    )))
}

pub async fn get_hash_render(
    context: PageContext,
    hash: String,
    per_page: i32,
    page: Option<i32>,
) -> Result<Response, Error> {
    render_threads(context, Some(OldIdOrHash::Hash(hash)), per_page, page).await
}

pub async fn get_fcid_render(
    context: PageContext,
    fcid: i64,
    per_page: i32,
    page: Option<i32>,
) -> Result<Response, Error> {
    render_threads(context, Some(OldIdOrHash::Id(fcid)), per_page, page).await
}
