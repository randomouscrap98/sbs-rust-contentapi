
use std::collections::HashMap;

use contentapi::{conversion::*, User};
use contentapi::endpoints::ApiContext;

use crate::_forumsys::*;

use super::*;


pub fn render(data: MainLayoutData, category: ForumCategory, path: Vec<ForumPathItem>, pages: Vec<ForumPagelistItem>) -> String {
    layout(&data, html!{
        section {
            h1 { (s(&category.category.name)) }
            (forum_path(&data.config, &path))
        }
        section {
            //Assume the stickies list is correct, they always come first no matter what
            @for sticky in &category.stickies {
                (thread_item(&data.config, sticky, &category.users))
                hr."smaller";
            }
            //Only care about 'unless' in the main list, the only time this DOES work is if there are ONLY stickies
            @for (index,thread) in category.threads.iter().enumerate() {
                (thread_item(&data.config, thread, &category.users))
                @if index < category.threads.len() - 1 {
                    hr."smaller";
                }
            }
            div."smallseparate" #"pagelist" {
                @for page in pages {
                    a."current"[page.current] href={(forum_category_link(&data.config, &category.category))"?page="(page.page)} { (page.text) }
                }
            }
        }
    }).into_string()
}

pub fn thread_item(config: &LinkConfig, thread: &ForumThread, users: &HashMap<i64, User>) -> Markup {
    html! {
        div."thread" {
            div."threadinfo" {
                h3 { a."flatlink" href=(forum_thread_link(config, &thread.thread)) { (s(&thread.thread.name)) } }
            }
            div."foruminfo aside mediumseparate" {
                (threadicon(config, thread))
                div { b { "Posts:" } (i(&thread.thread.commentCount.into())) } //{{thread.commentCount}}</div>
                div {
                    b { "Created:" }
                    time datetime=(d(&thread.thread.createDate)) { (timeago_o(&thread.thread.createDate)) }
                }
                @if let Some(post) = thread.posts.get(0) {
                    div {
                        b { "Last:" }
                        a."flatlink" href=(forum_post_link(config, post, &thread.thread)) {
                            time datetime=(d(&post.createDate)) { (timeago_o(&post.createDate)) }
                        }
                        " by "
                        @if let Some(user_id) = post.createUserId {
                            @if let Some(user) = users.get(&user_id) {
                                a."flatlink" href=(user_link(config, user)) { (user.username) }
                            }
                        }
                    }
                }
            }
        }
    }
}
