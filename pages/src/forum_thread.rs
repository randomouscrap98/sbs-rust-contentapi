
use std::collections::HashMap;

use bbcode::BBCode;

use crate::_forumsys::*;

use super::*;

pub fn render(data: MainLayoutData, bbcode: &BBCode, thread: ForumThread, users: &HashMap<i64,User>, path: Vec<ForumPathItem>,
    pages: Vec<ForumPagelistItem>, selected_post_id: Option<i64>) -> String 
{
    layout(&data, html!{
        (style(&data.config, "/forum.css"))
        section {
            h1 { (s(&thread.thread.name)) }
            (forum_path(&data.config, &path))
            div."foruminfo smallseparate aside" {
                (threadicon(&data.config, &thread))
                span {
                    b { "OP:" }
                    @if let Some(user) = users.get(&thread.thread.createUserId.unwrap_or(0)) {
                        a."flatlink" href=(user_link(&data.config, user)){ (user.username) }
                    }
                }
                span {
                    b { "Created:" }
                    time datetime=(d(&thread.thread.createDate)) { (timeago_o(&thread.thread.createDate)) }
                }
            }
        }
        section data-selected=[selected_post_id] {
            @for (index,post) in thread.posts.iter().enumerate() {
                (post_item(&data.config, bbcode, post, &thread.thread, selected_post_id, users))
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
    users: &HashMap<i64, User>) -> Markup 
{
    let user = user_or_default(users.get(&post.createUserId.unwrap_or(0)));
    let mut class = String::from("post");
    if selected_post_id == post.id { class.push_str(" current") }
    html! {
        div.(class) #{"post_"(i(&post.id))} {
            div."postleft" {
                img."avatar" src=(image_link(config, &user.avatar, 100, true)); //"{{imagelink user.avatar 100 true}}">
            }
            div."postright" {
                div."postheader" {
                    a."flatlink username" href=(user_link(config, &user)) { (&user.username) } 
                    a."sequence" href=(forum_post_link(config, post, thread)){ "#" (123) } //{{lookup @root.sequence (string ../id)}}</a>
                }
                @if let Some(text) = &post.text {
                    div."content bbcode" { (PreEscaped(bbcode.parse(text))) }
                }
                div."postfooter" {
                    div."history" {
                        time."aside" datetime=(d(&post.createDate)) { (timeago_o(&post.createDate)) } //{timeago ../createDate}}</time>
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
