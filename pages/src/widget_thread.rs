use common::*;

use std::collections::HashMap;

use contentapi::*;
use common::forum::*;
use common::submission::*; //Again, controversial? idk
use common::pagination::*;
use common::layout::*;
use bbscope::BBCode;
use maud::*;


pub struct PostsConfig {
    /// The thread that holds all the posts to render
    pub thread: ForumThread,
    pub users: HashMap<i64,User>,
    /// The path to this thread; if not given, path not rendered. Thread must also be given
    pub path: Option<Vec<ForumPathItem>>,
    /// The pages to navigate posts; not displayed if not given
    pub pages: Option<Vec<PagelistItem>>,
    pub start_num: i32,
    pub selected_post_id: Option<i64>,

    pub render_header: bool,
    pub render_page: bool
}

/// Rendering for the actual widget. The 
pub fn render(context: &mut PageContext, config: PostsConfig) -> String {
    let posts = render_posts(context, config);
    html!{
        (DOCTYPE)
        html lang=(context.layout_data.user_config.language) {
            head {
                (basic_meta(&context.layout_data.config))
                title { "SmileBASIC Source Image Browser" }
                meta name="description" content="Simple image browser widget";
                (style(&context.layout_data.config, "/base.css"))
                (script(&context.layout_data.config, "/base.js"))
            }
            //This is meant to go in an iframe, so it will use up the whole space
            (body(&context.layout_data, html! {
                (posts)
            }))
        }
    }.into_string()
}

pub fn render_posts(context: &mut PageContext, config: PostsConfig) -> Markup
{
    let data = &context.layout_data;
    let bbcode = &mut context.bbcode;
    let thread = &config.thread;
    html!{
        (style(&data.config, "/forpage/forum.css"))
        @if config.render_header {
            section {
                h1 { (s(&thread.thread.name)) }
                @if let Some(path) = config.path {
                    (forum_path(&data.config, &path))
                }
                div."foruminfo smallseparate aside" {
                    (threadicon(&data.config, &thread))
                    span {
                        /*b { "By: " }*/
                        @if let Some(user) = config.users.get(&thread.thread.createUserId.unwrap_or(0)) {
                            a."flatlink" href=(user_link(&data.config, user)){ (user.username) }
                        }
                    }
                    span {
                        b { "Created: " }
                        time datetime=(d(&thread.thread.createDate)) { (timeago_o(&thread.thread.createDate)) }
                    }
                }
            }
        }
        @if config.render_page && thread.thread.literalType != Some(SBSContentType::forumthread.to_string()) {
            (render_page(&data, bbcode, &thread))
        }
        section #"thread-top" data-selected=[config.selected_post_id] {
            @for (index,post) in thread.posts.iter().enumerate() {
                (post_item(&data.config, bbcode, post, &thread.thread, config.selected_post_id, &config.users, config.start_num + index as i32))
                @if index < thread.posts.len() - 1 { hr."smaller"; }
            }
            @if let Some(pages) = config.pages {
                div."smallseparate pagelist" {
                    @for page in pages {
                        a."current"[page.current] href={(forum_thread_link(&data.config, &thread.thread))"?page="(page.page)"#thread-top"} { (page.text) }
                    }
                }
            }
        }
    }
}

/// Render the page data, such as text and infoboxes, on standard pages. True forum threads don't have main
/// content like that, so this is only called on programs, resources, etc
pub fn render_page(data: &MainLayoutData, bbcode: &mut BBCode, thread: &ForumThread) -> Markup 
{
    let values = match &thread.thread.values { Some(values) => values.clone(), None => HashMap::new() };
    html!{
        section {
            //First check is if it's a program, then we float this box to the right
            @if thread.thread.literalType == Some(SBSContentType::program.to_string()) {
                div."programinfo" {
                    @if let Some(images) = values.get(IMAGESKEY).and_then(|k| k.as_array()) {
                        div."gallery" {
                            //we now have the images: we just need the first one (it's a hash?)
                            @if let Some(image) = images.get(0).and_then(|i| i.as_str()) {
                                img src=(base_image_link(&data.config, image));
                            }
                        }
                    }
                    div."extras mediumseparate" {
                        @if let Some(key) = values.get(DOWNLOADKEYKEY).and_then(|k| k.as_str()) {
                            span."smallseparate" {
                                b { "Download:" }
                                span."key" { (key) }
                                (threadicon(&data.config, &thread))
                            }
                        }
                        @if let Some(version) = values.get(VERSIONKEY).and_then(|k| k.as_str()) {
                            span."smallseparate" {
                                b { "Version:" }
                                span."version" { (version) }
                            }
                        }
                        @if let Some(size) = values.get(SIZEKEY).and_then(|k| k.as_str()) {
                            span."smallseparate" {
                                b { "Size:" }
                                span."size" { (size) }
                            }
                        }
                    }
                }
            }
            //Next check is if there's even any text to show
            @if let Some(text) = &thread.thread.text {
                div."content bbcode" { (PreEscaped(&bbcode.parse_profiled_opt(&text, format!("program-{}", i(&thread.thread.id))))) }
            }
        }
    }
}

/// Render a single post on a thread.
pub fn post_item(config: &LinkConfig, bbcode: &mut BBCode, post: &Message, thread: &Content, selected_post_id: Option<i64>, 
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
                    a."sequence" target="_top" title=(i(&post.id)) href=(forum_post_link(config, post, thread)){ "#" (sequence) } 
                }
                @if let Some(text) = &post.text {
                    div."content bbcode" { (PreEscaped(bbcode.parse_profiled_opt(text, format!("post-{}",i(&post.id))))) }
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