
use std::collections::HashMap;

use bbcode::BBCode;
use contentapi::{endpoints::ApiContext, FullRequest};

use contentapi::{conversion::*, SpecialCount};
use crate::_forumsys::*;

use super::*;

pub fn render(data: MainLayoutData, bbcode: &BBCode, thread: ForumThread, users: &HashMap<i64,User>, path: Vec<ForumPathItem>,
    pages: Vec<ForumPagelistItem>, start_num: i32, selected_post_id: Option<i64>) -> String 
{
    layout(&data, html!{
        (style(&data.config, "/forum.css"))
        section {
            h1 { (s(&thread.thread.name)) }
            (forum_path(&data.config, &path))
            div."foruminfo smallseparate aside" {
                (threadicon(&data.config, &thread))
                span {
                    b { "OP: " }
                    @if let Some(user) = users.get(&thread.thread.createUserId.unwrap_or(0)) {
                        a."flatlink" href=(user_link(&data.config, user)){ (user.username) }
                    }
                }
                span {
                    b { "Created: " }
                    time datetime=(d(&thread.thread.createDate)) { (timeago_o(&thread.thread.createDate)) }
                }
            }
        }
        section data-selected=[selected_post_id] {
            @for (index,post) in thread.posts.iter().enumerate() {
                (post_item(&data.config, bbcode, post, &thread.thread, selected_post_id, users, start_num + index as i32))
                @if index < thread.posts.len() - 1 { hr."smaller"; }
            }
            div."smallseparate" #"pagelist" {
                @for page in pages {
                    a."current"[page.current] href={(forum_thread_link(&data.config, &thread.thread))"?page="(page.page)} { (page.text) }
                }
            }
        }
    }).into_string()
}

fn post_item(config: &LinkConfig, bbcode: &BBCode, post: &Message, thread: &Content, selected_post_id: Option<i64>, 
    users: &HashMap<i64, User>, sequence: i32) -> Markup 
{
    let user = user_or_default(users.get(&post.createUserId.unwrap_or(0)));
    let mut class = String::from("post");
    if selected_post_id == post.id { class.push_str(" current") }
    html! {
        div.(class) #{"post_"(i(&post.id))} {
            div."postleft" {
                img."avatar" src=(image_link(config, &user.avatar, 100, true)); 
            }
            div."postright" {
                div."postheader" {
                    a."flatlink username" href=(user_link(config, &user)) { (&user.username) } 
                    a."sequence" href=(forum_post_link(config, post, thread)){ "#" (sequence) } 
                }
                @if let Some(text) = &post.text {
                    div."content bbcode" { (PreEscaped(bbcode.parse(text))) }
                }
                div."postfooter" {
                    div."history" {
                        time."aside" datetime=(d(&post.createDate)) { (timeago_o(&post.createDate)) } 
                        @if let Some(edit_user_id) = post.editUserId {
                            time."aside" datetime=(d(&post.editDate)) { 
                                "Edited "(timeago_o(&post.editDate))" by "
                                @if let Some(edit_user) = users.get(&edit_user_id) {
                                    a."flatlink" href=(user_link(config,&edit_user)){ (&edit_user.username) }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}


async fn render_thread(data: MainLayoutData, context: &ApiContext, bbcode: &BBCode, pre_request: FullRequest, per_page: i32, 
    page: Option<i32>) -> Result<Response, Error> 
{
    let mut page = page.unwrap_or(1) - 1; //we assume 1-based pages

    //Go lookup all the 'initial' data, which everything except posts and users
    let pre_result = context.post_request(&pre_request).await?;

    //Pull out and parse all that stupid data. It's fun using strongly typed languages!! maybe...
    let mut categories_cleaned = CleanedPreCategory::from_many(cast_result_required::<Content>(&pre_result, CATEGORYKEY)?)?;
    let mut threads_raw = cast_result_required::<Content>(&pre_result, THREADKEY)?;
    let selected_post = cast_result_safe::<Message>(&pre_result, PREMESSAGEKEY)?.pop();
    if let Some(message_index) = cast_result_safe::<SpecialCount>(&pre_result, PREMESSAGEINDEXKEY)?.pop() {
        //The index is the special count. This means we change the page given. If page wasn't already 0, we warn
        if page != 0 {
            println!("Page was nonzero ({}) while there was a message index ({})", page, message_index.specialCount);
        }
        page = message_index.specialCount / per_page;
    }

    //There must be one category, and one thread, otherwise return 404
    let thread = threads_raw.pop().ok_or(Error::NotFound(String::from("Could not find thread!")))?;
    let category = categories_cleaned.pop().ok_or(Error::NotFound(String::from("Could not find category!")))?;

    //Also I need some fields to exist.
    let thread_id = thread.id.ok_or(Error::Other(String::from("Thread result did not have id field?!")))?;
    let thread_create_uid = thread.createUserId.ok_or(Error::Other(String::from("Thread result did not have createUserId field!")))?;
    let comment_count = thread.commentCount.ok_or(Error::Other(String::from("Thread result did not have commentCount field!")))?;

    let sequence_start = page * per_page; 

    //OK NOW you can go lookup the posts, since we are sure about where in the postlist we want
    let after_request = get_finishpost_request(thread_id, vec![thread_create_uid], 
        per_page, sequence_start);
        //context.config.default_display_posts, sequence_start);
    let after_result = context.post_request(&after_request).await?;

    //Pull the data out of THAT request
    let messages_raw = cast_result_required::<Message>(&after_result, "message")?;
    let users_raw = cast_result_required::<User>(&after_result, "user")?;

    //Construct before borrowing 
    let path = vec![ForumPathItem::root(), ForumPathItem::from_category(&category.category), ForumPathItem::from_thread(&thread)];
    Ok(Response::Render(render(
        data, 
        bbcode, 
        ForumThread::from_content(thread, &messages_raw, &category.stickies)?, 
        &users_raw.into_iter().map(|u| (u.id, u)).collect::<HashMap<i64, User>>(),
        path,
        get_pagelist(comment_count as i32, per_page, page),
        1 + per_page * page,
        selected_post.and_then(|m| m.id)
    )))
}

//#[get("/forum/thread/<hash>/<post_id>")]

/// The normal endpoint for listing a thread
pub async fn get_hash_render(data: MainLayoutData, context: &ApiContext, bbcode: &BBCode, hash: String, 
    per_page: i32, page: Option<i32>) -> Result<Response, Error> 
{
    render_thread(
        data, context, bbcode, 
        get_prepost_request(None, None, None, Some(hash)), 
        per_page, page).await
}

/// The normal endpoint for pinpointing a post
pub async fn get_hash_postid_render(data: MainLayoutData, context: &ApiContext, bbcode: &BBCode, hash: String, post_id: i64,
    per_page: i32) -> Result<Response, Error> 
{
    render_thread(
        data, context, bbcode, 
        get_prepost_request(None, Some(post_id), None, Some(hash)), 
        per_page, None).await
}

//#[get("/forum?<ftid>&<page>", rank=5)] //, rank=3)]
pub async fn get_ftid_render(data: MainLayoutData, context: &ApiContext, bbcode: &BBCode, ftid: i64, 
    per_page: i32, page: Option<i32>) -> Result<Response, Error> 
{
    render_thread(
        data, context, bbcode, 
        get_prepost_request(None, None, Some(ftid), None), 
        per_page, page).await
}

//Most old links may be to posts directly? idk
//#[get("/forum?<fpid>", rank=3)] //, rank=4)]
pub async fn get_fpid_render(data: MainLayoutData, context: &ApiContext, bbcode: &BBCode, fpid: i64,
    per_page: i32) -> Result<Response, Error> 
{
    render_thread(
        data, context, bbcode, 
        get_prepost_request(Some(fpid), None, None, None), 
        per_page, None).await
}