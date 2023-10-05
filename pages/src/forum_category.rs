
use std::collections::HashMap;

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
use common::response::*;
use contentapi::permissions::can_user_action;
use maud::*;


pub fn render(mut data: MainLayoutData, category: ForumCategory, path: Vec<ForumPathItem>, pages: Vec<PagelistItem>) -> String 
{
    let mut can_create_threads = false; 

    if category.category.literalType.as_deref() == Some(SBSPageType::SUBMISSIONS) {
        data.override_nav_path = Some("/search");
    }
    else if category.category.literalType.as_deref() == Some(SBSPageType::DIRECTMESSAGES) {
        data.override_nav_path = Some("/userhome");
    }
    else { //This is a normal category, so we might be able to create threads
        if let Some(ref user) = data.user {
            can_create_threads = can_user_action(user, "C", &category.category);
        }
    }

    layout(&data, html!{
        (data.links.style("/forpage/forum.css"))
        section {
            h1 { (opt_s!(category.category.name)) }
            p."aside" {(opt_s!(category.category.description))}
            (forum_path(&data.links, &path))
        }
        section {
            //Assume the stickies list is correct, they always come first no matter what
            @for sticky in &category.stickies {
                (thread_item(&data.links, sticky, &category.users))
                hr."smaller";
            }
            //Only care about 'unless' in the main list, the only time this DOES work is if there are ONLY stickies
            @for (index,thread) in category.threads.iter().enumerate() {
                (thread_item(&data.links, thread, &category.users))
                @if index < category.threads.len() - 1 {
                    hr."smaller";
                }
            }
            div."smallseparate pagelist" {
                @for page in pages {
                    a."current"[page.current] href={(data.links.forum_category(&category.category))"?page="(page.page)} { (page.text) }
                }
            }
            //Not sure if we should re-use pagelist, probably bad
            @if can_create_threads {
                div."smallseparate pagelist" {
                    a."coolbutton" #"newthread" href=(data.links.forum_thread_editor_new(&category.category)) { "New thread" }
                }
            }
        }
    }).into_string()
}

pub fn thread_item(links: &LinkConfig, thread: &ForumThread, users: &HashMap<i64, User>) -> Markup {
    html! {
        div."thread" {
            div."threadinfo" {
                h3 { a."flatlink" href=(links.forum_thread(&thread.thread)) { (opt_s!(thread.thread.name, "??? (NOTITLE)")) } }
            }
            div."foruminfo aside mediumseparate" {
                (threadicon(links, thread))
                div { b { "Posts: " } (i(&thread.thread.commentCount.into())) }
                div {
                    b { "Created: " }
                    time datetime=(d(&thread.thread.createDate)) { (timeago_o(&thread.thread.createDate)) }
                }
                @if let Some(post) = thread.posts.get(0) {
                    div {
                        b { "Last: " }
                        a."flatlink" href=(links.forum_post(post, &thread.thread)) {
                            time datetime=(d(&post.createDate)) { (timeago_o(&post.createDate)) }
                        }
                        " by "
                        @if let Some(user_id) = post.createUserId {
                            @if let Some(user) = users.get(&user_id) {
                                a."flatlink" href=(links.user(user)) { (user.username) }
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn build_categories_with_threads(context: &mut ApiContext, categories_cleaned: Vec<CleanedPreCategory>, limit: i32, skip: i32) -> 
    Result<Vec<ForumCategory>, Error> 
{
    //Next request: get the complicated dataset for each category (this somehow includes comments???)
    let thread_request = get_thread_request(&categories_cleaned, limit, skip, true); //context.config.default_category_threads, 0);
    let thread_result = context.post_request_profiled_opt(&thread_request, "getthreads").await?;

    let messages_raw = cast_result_required::<Message>(&thread_result, "message")?;

    let mut categories = Vec::new();

    for category in categories_cleaned {
        categories.push(ForumCategory::from_result(category, &thread_result, &messages_raw)?);
    }

    Ok(categories)
}

async fn render_threads(mut context: PageContext, category_request: FullRequest, per_page: i32, page: Option<i32>) ->
    Result<Response, Error>
{
    let page = page.unwrap_or(1) - 1;

    let category_result = context.api_context.post_request_profiled_opt(&category_request, "getcategory").await?;
    let categories_cleaned = CleanedPreCategory::from_many(cast_result_required::<Content>(&category_result, CATEGORYKEY)?)?;
    let mut categories = build_categories_with_threads(&mut context.api_context, categories_cleaned, 
        per_page,
        page * per_page
    ).await?;

    //TODO: Might want to add data to these RouteErrors?
    let category = categories.pop().ok_or(Error::NotFound(String::from("Couldn't find that category")))?;
    let pagelist = get_pagelist(category.threads_count, per_page, page);

    //println!("Please: {:?}", category);

    let path = vec![ForumPathItem::root(), ForumPathItem::from_category(&category.category)];
    Ok(Response::Render(render(context.layout_data, category, path, pagelist)))
}



pub async fn get_hash_render(context: PageContext, hash: String, per_page: i32, page: Option<i32>) -> 
    Result<Response, Error> 
{
    render_threads(context, get_category_request(Some(hash), None), per_page, page).await
}

pub async fn get_fcid_render(context: PageContext, fcid: i64, per_page: i32, page: Option<i32>) -> 
    Result<Response, Error> 
{
    render_threads(context, get_category_request(None, Some(fcid)), per_page, page).await
}