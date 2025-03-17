use std::collections::HashMap;

use contentapi::*;
use maud::*;
// use serde_json::Value;

use crate::capi::DocTreeContent;
use crate::capi::ForumCategory2;
use crate::capi::ForumThread2;
use crate::constants::*;
use crate::forms::*;
use crate::forum::*;
use crate::pagination::*;
use crate::render::*;
//use crate::response::Error;
use crate::view::*;
use crate::*;

// ----------------------------
// *       BASIC JUNK         *
// ----------------------------

//To build the forum path at the top
pub struct ForumPathItem {
    pub link: String,
    pub title: String,
}

impl ForumPathItem {
    pub fn from_category2(category: &ForumCategory2) -> Self {
        Self {
            link: format!("/forum/category/{}", &category.hash),
            title: category.name.clone(),
        }
    }
    // pub fn from_category(category: &Content) -> Self {
    //     Self {
    //         link: format!("/forum/category/{}", opt_s!(category.hash)),
    //         title: String::from(opt_s!(category.name, "NOTFOUND")),
    //     }
    // }
    pub fn from_thread2(thread: &ForumThread2) -> Self {
        Self {
            link: format!("/forum/thread/{}", thread.hash),
            title: thread.name.clone(),
        }
    }
    // pub fn from_thread(thread: &Content) -> Self {
    //     Self {
    //         link: format!("/forum/thread/{}", opt_s!(thread.hash)),
    //         title: String::from(opt_s!(thread.name, "NOTFOUND")),
    //     }
    // }
    pub fn root() -> Self {
        Self {
            link: String::from("/forum"),
            title: String::from("Root"),
        }
    }
}

pub fn forum_path(config: &LinkConfig, path: &Vec<ForumPathItem>) -> Markup {
    html! {
        p."forumpath" {
            @for (index, segment) in path.iter().enumerate() {
                @let last = index == path.len() - 1;
                a."flatlink" href={(config.http_root)(segment.link)} {
                    @if last { "[.]" }
                    @else { (segment.title) }
                }
                @if !last {
                    span."pathseparator" { " / " }
                }
            }
        }
    }
}

//Weird circular dependency... oh well, maybe I'll fix later
// pub fn threadicon(config: &LinkConfig, thread: &ForumThread) -> Markup {
//     html! {
//         div."threadicon smallseparate" {
//             (render::submissions::pageicon(config, &thread.thread))
//         }
//     }
// }

pub fn threadicon2(config: &LinkConfig, thread: &ForumThread2) -> Markup {
    html! {
        div."threadicon smallseparate" {
            (render::submissions::pageicon2(config, &thread.values, &thread.literal_type, 99))
        }
    }
}

// ----------------------------
// *     BIG JUNK (THReAD)    *
// ----------------------------

/// Render a single post on a thread.
pub struct PostsConfig {
    /// The thread that holds all the posts to render
    pub thread: ForumThread2,
    pub categories: Vec<capi::SubmissionCategory>,
    pub posts: Vec<Message>,
    pub related: HashMap<i64, Message>,
    pub users: HashMap<i64, User>,
    /// The path to this thread; if not given, path not rendered. Thread must also be given
    pub path: Option<Vec<ForumPathItem>>,
    /// The pages to navigate posts; not displayed if not given
    pub pages: Option<Vec<PagelistItem>>,
    pub start_num: Option<i32>,
    pub selected_post_id: Option<i64>,
    //pub docs_content: Option<Vec<Content>>, //DocTreeNode<'a>>,
    pub render_header: bool,
    pub render_page: bool,
    pub render_reply_chain: bool,
    pub render_reply_link: bool,
    pub render_controls: bool, //pub render_sequence: bool
}

impl PostsConfig {
    pub fn thread_mode(
        thread: ForumThread2,
        categories: Vec<capi::SubmissionCategory>,
        posts: Vec<Message>,
        related: HashMap<i64, Message>,
        users: HashMap<i64, User>,
        path: Vec<ForumPathItem>,
        pages: Vec<PagelistItem>,
        start: i32,
        selected_post_id: Option<i64>,
    ) -> Self {
        //let posts = thread.posts.clone();
        Self {
            thread,
            related,
            users,
            posts,
            categories,
            path: Some(path),
            pages: Some(pages),
            start_num: Some(start),
            selected_post_id,
            render_header: true,
            render_page: true,
            render_reply_chain: false,
            render_reply_link: true,
            render_controls: true,
            //docs_content: None,
        }
    }
    pub fn reply_mode(
        thread: ForumThread2,
        posts: Vec<Message>,
        related: HashMap<i64, Message>,
        users: HashMap<i64, User>,
        selected_post_id: Option<i64>,
    ) -> Self {
        //let posts = thread.posts.clone();
        Self {
            thread,
            related,
            users,
            posts,
            categories: Vec::new(),
            path: None,
            pages: None,
            start_num: None,
            selected_post_id,
            render_header: false,
            render_page: false,
            render_reply_chain: true,
            render_reply_link: false,
            render_controls: false,
            //docs_content: None,
        }
    }
}

fn walk_post_tree(
    layout_data: &MainLayoutData,
    bbcode: &mut BBCode,
    config: &PostsConfig,
    tree: &ReplyTree,
    sequence: Option<i32>,
    posts_left: &mut i32,
) -> Markup {
    *posts_left -= 1;
    html! {
        //@let (sequence = config.start_num.and_then(|s| Some(s + index as i32));
        (post_item(layout_data, bbcode, config, tree.post, sequence))
        @if *posts_left > 0 { hr."smaller"; }
        @if tree.children.len() > 0 {
            div."replychain" {
                @for child in &tree.children {
                    //Note: only the very top level should get sequence numbers, so all inner recursive calls get None sequence
                    //@let (markup, posts_left) = (walk_post_tree(layout_data, &mut bbcode, config, child, None, posts_left - 1))
                    (walk_post_tree(layout_data, bbcode, config, child, None, posts_left))
                }
            }
        }
    }
}

/// Render the main sections of a content and message stream (the MAIN view on the website!) but configured
/// for the particular viewing instance. WARN: THIS ALSO MODIFIES context WITH APPROPRIATE OVERRIDES! A bit
/// more than rendering, I guess...
pub fn render_mainview(context: &mut PageContext, config: PostsConfig) -> Markup {
    let thread = &config.thread;
    let thread_type = &thread.literal_type; //.as_deref();

    let is_pagetype = thread_type == SBSPageType::PROGRAM
        || thread_type == SBSPageType::RESOURCE
        || thread_type == SBSPageType::DOCUMENTATION;

    //Here, we choose how the override works
    if thread_type == SBSPageType::PROGRAM || thread_type == SBSPageType::RESOURCE {
        context.layout_data.override_nav_path = Some("/search");
    } else if thread_type == SBSPageType::DOCUMENTATION {
        context.layout_data.override_nav_path = Some("/documentation");
    } else if thread_type == SBSPageType::DIRECTMESSAGE {
        context.layout_data.override_nav_path = Some("/userhome");
    }

    let data = &context.layout_data;
    let bbcode = &mut context.bbcode;
    let mut post_count = config.posts.len() as i32;

    let reply_tree: Vec<ReplyTree> = if config.render_reply_chain {
        posts_to_replytree(&config.posts)
    } else {
        // no reply chain is just a simple list of whatever
        config.posts.iter().map(|m| ReplyTree::new(m)).collect()
    };

    let mut pagelist_html: Option<Markup> = None;
    if let Some(ref pages) = config.pages {
        if pages.len() > 1 {
            pagelist_html = Some(html! {
                div."smallseparate pagelist" {
                    @for page in pages {
                        a."current"[page.current] target="_top" href={(data.links.forum_thread_unsafe(&thread.hash))"?page="(page.page)"#thread-top"} { (page.text) }
                    }
                }
            })
        }
    }

    html! {
        (data.links.style("/forpage/forum.css"))
        (data.links.script("/forpage/forum.js"))
        @if config.render_header {
            section {
                h1 title=(thread.id) { (thread.name) }
                @if let Some(path) = &config.path {
                    (forum_path(&data.links, &path))
                }
                div."foruminfo smallseparate aside" {
                    (threadicon2(&data.links, &thread))
                    //Snail doesn't want the create user displayed on documentation
                    @if thread_type != SBSPageType::DOCUMENTATION {
                        span {
                            @if let Some(user) = config.users.get(&thread.create_user_id) {
                                a."flatlink" target="_top" href=(data.links.user(user)){ (user.username) }
                            }
                        }
                    }
                    span {
                        b { "Created: " }
                        time datetime=(dd(&thread.create_date)) { (timeago(&thread.create_date)) }
                    }
                    iframe."votes" src={(data.links.votewidget_unsafe(thread.id))}{}
                }
            }
        }
        @if config.render_page && is_pagetype {
            (render_page(&data, bbcode, &thread, &config.categories)) //, &config.docs_content))
        }
        //it says "thread-top" because it is: it's the beginning of the section that displays posts. After the
        //for loop, it then displays pages, which is on the bottom of the thread, so it might seem confusing.
        //maybe the id should be changed to an anchor, idr how to do that.
        section #"thread-top" data-selected=[config.selected_post_id] {
            @if data.user_config.toppagination_posts {
                @if let Some(ref pagelist) = pagelist_html {
                    (pagelist)
                    hr."smaller";
                }
            }
            @if reply_tree.len() > 0 {
                @for (index,tree) in reply_tree.iter().enumerate() {
                    @let sequence = config.start_num.and_then(|s| Some(s + index as i32));
                    (walk_post_tree(&context.layout_data, &mut context.bbcode, &config, tree, sequence, &mut post_count))
                }
            }
            @else {
                //As usual, I'm reusing pagelist to make centered and spaced content
                p."aside pagelist" { "No posts yet (will you be the first?)" }
            }
            @if let Some(ref pagelist) = pagelist_html {
                (pagelist)
            }
        }
    }
}

fn images_to_attr(config: &LinkConfig, images: &Vec<String>) -> String {
    serde_json::to_string(
        &images
            .iter()
            .map(|i| config.image_default(i))
            .collect::<Vec<String>>(),
    )
    .unwrap_or_else(|err| {
        println!("ERROR: COULD NOT SERIALIZE PAGE IMAGES: {}", err);
        String::new()
    })
}

fn walk_doctree_recursive(
    layout_data: &MainLayoutData,
    tree: &DocTreeNode,
    open_levels: i32,
) -> Markup {
    let mut tree_nodes = tree.tree_nodes.clone();
    tree_nodes.sort_by(|a, b| a.name.cmp(&b.name));

    html! {
        details."docnode" open[open_levels > 0] {
            summary { (tree.name) }
            @for node in &tree_nodes {
                (walk_doctree_recursive(layout_data, node, open_levels - 1))
            }
            ul {
                @for content in &tree.page_nodes {
                    li { a."flatlink" href=(layout_data.links.forum_thread_unsafe(&content.hash)) { (content.name) } }
                }
            }
        }
    }
}

fn walk_doctree(layout_data: &MainLayoutData, tree: &DocTreeNode, open_levels: i32) -> Markup {
    let mut tree_nodes = tree.tree_nodes.clone();
    tree_nodes.sort_by(|a, b| a.name.cmp(&b.name));

    //For the root node, we only list the treenodes and NOT ourselves. Then normal recursion
    html! {
        @for node in &tree_nodes {
            (walk_doctree_recursive(layout_data, node, open_levels))
        }
    }
}

pub fn display_doctree(
    layout_data: &MainLayoutData,
    documentation: Vec<DocTreeContent>,
    open_levels: i32,
) -> Markup {
    html! {
        div."documenttree" {
            (walk_doctree(layout_data, &mut get_doctree(&documentation), open_levels))
        }
    }
}

/// Render the page data, such as text and infoboxes, on standard pages. True forum threads don't have main
/// content like that, so this is only called on programs, resources, etc
pub fn render_page(
    data: &MainLayoutData,
    bbcode: &mut BBCode,
    thread: &ForumThread2,
    categories: &Vec<capi::SubmissionCategory>,
    //_docs_content: &Option<Vec<Content>>,
) -> Markup {
    let values = &thread.values;
    let systems = get_systems2(&values);

    let images: Option<Vec<String>> = get_value_safe!(values, SBSValue::IMAGES, Vec<String>);
    let dlkey: Option<String> = get_value_safe!(values, SBSValue::DOWNLOADKEY, String);
    let version: Option<String> = get_value_safe!(values, SBSValue::VERSION, String);
    let size: Option<String> = get_value_safe!(values, SBSValue::SIZE, String);

    html! {
        section {
            //First check is if it's a program, then we float this box to the right
            @if thread.literal_type == SBSPageType::PROGRAM {
                div."programinfo" {
                    @if let Some(images) = images {
                        div."gallery" #"page_gallery" /*data-index="0"*/ data-images=(images_to_attr(&data.links, &images)) {
                            //we now have the images: we just need the first one (it's a hash?)
                            @if let Some(image) = images.get(0) {
                                img src=(data.links.image_default(image));
                            }
                        }
                    }
                    div."extras mediumseparate" {
                        @if let Some(key) = dlkey {
                            span."smallseparate" {
                                b { "Download:" }
                                span."key" { (key) }
                                (threadicon2(&data.links, &thread))
                            }
                        }
                        @if systems.iter().any(|s| s == &PTCSYSTEM) {
                            span."smallseparate" {
                                b { "Download:" }
                                a."key" href=(data.links.qr_generator_unsafe(&thread.hash)) { "QR Codes" }
                                (threadicon2(&data.links, &thread))
                            }
                        }
                        @if let Some(version) = version {
                            span."smallseparate" {
                                b { "Version:" }
                                span."version" { (version) }
                            }
                        }
                        @if let Some(size) = size {
                            span."smallseparate" {
                                b { "Size:" }
                                span."size" { (size) }
                            }
                        }
                    }
                }
            }
            //Snail says he doesn't want the doctree on pages
            //@if thread.thread.literalType.as_deref() == Some(SBSPageType::DOCUMENTATION) {
            //    @if let Some(docs) = docs_content {
            //        (display_doctree(data, docs, 0))
            //    }
            //    @else {
            //        div."error" {
            //            ({
            //                println!("Tried to render documentation without a doctree!");
            //                "NO DOCTREE FOUND!"
            //            })
            //        }
            //    }
            //}
            (render_textcontent(&thread.text, &thread.values, bbcode))
            //Documentation has no categories
            @if thread.literal_type != SBSPageType::DOCUMENTATION {
                hr."smaller";
                div."categorylist smallseparate" {
                    @for category in categories {
                        a."flatlink" href=(data.links.search_category(category.id)) { (category.name) }
                    }
                }
            }
        }
    }
}

//Now that we support multiple markups, rendering content can get a little complex
pub fn render_textcontent(
    text: &str,
    values: &HashMap<String, String>,
    bbcode: &mut BBCode,
) -> Markup {
    let mut markup: &str = MARKUPBBCODE;
    if let Some(mkup) = get_value_safe!(values, SBSValue::MARKUP, &str) {
        markup = mkup;
    }
    // if let Some(ref values) = content.values {
    //     if let Some(mk) = values.get(SBSValue::MARKUP) {
    //         if let Some(mk) = mk.as_str() {
    //             markup = mk;
    //         }
    //     }
    // }
    html!(
        div."content" data-markup=(markup) data-prerendered[markup == MARKUPBBCODE] {
            @if markup == MARKUPBBCODE {
                (PreEscaped(&bbcode.parse(text)))
            }
            @else {
                (text)
            }
        }
    )
}

/// Render content WITHOUT a full content. This is more expensive than just rendering with content (sorry?)
// pub fn render_content_nocontent(
//     text: String,
//     markup: Option<String>,
//     bbcode: &mut BBCode,
// ) -> Markup {
//     let mut content = Content::default();
//     content.text = Some(text);
//     if let Some(markup) = markup {
//         let mut values: HashMap<String, Value> = HashMap::new();
//         values.insert(SBSValue::MARKUP.to_string(), markup.into());
//         content.values = Some(values);
//     }
//     render_content(&content, bbcode)
// }

//WAS consuming bbcode, now i'm not sure. leaving for now
pub fn post_item(
    layout_data: &MainLayoutData,
    bbcode: &mut BBCode,
    config: &PostsConfig,
    post: &Message,
    sequence: Option<i32>,
) -> Markup {
    let users = &config.users;
    let user = user_or_default(users.get(&post.createUserId.unwrap_or(0)));
    let mut class = String::from("post");
    if config.selected_post_id == post.id {
        class.push_str(" current")
    }
    let mut reply_chain_link: Option<String> = None;
    let mut reply_post: Option<&Message> = None;

    if let Some(replies) = get_replydata(post) {
        reply_post = config.related.get(&replies.direct);
        if reply_post.is_none() {
            println!("ERROR: couldn't find related post {}!", replies.direct)
        }
        if config.render_reply_link {
            let query = ThreadQuery {
                reply: Some(replies.top),
                selected: post.id,
            };
            match serde_urlencoded::to_string(query) {
                Ok(query) => {
                    reply_chain_link = Some(format!(
                        "{}/widget/thread?{}",
                        &layout_data.links.http_root, query
                    )); //, forum_post_hash(post)));
                }
                Err(error) => println!("ERROR: couldn't encode thread query!: {}", error),
            }
        }
    }

    html! {
        div.(class) #{"post_"(i(&post.id))} {
            div."postleft" {
                img."avatar" src=(layout_data.links.image(&user.avatar, QueryImage::Cropped100 ));
            }
            div."postright" {
                div."postheader" {
                    a."flatlink username" target="_top" href=(layout_data.links.user(&user)) { (&user.username) }
                    @if let Some(sequence) = sequence {
                        a."sequence" target="_top" title=(i(&post.id)) href=(layout_data.links.forum_post_unsafe(post.id.unwrap_or_default(), &config.thread.hash)){ "#" (sequence) }
                    }
                }
                @if let Some(reply_post) = reply_post {
                    //TODO: can't decide between consuming or not. spoilers are the important bit
                    (post_reply(layout_data, bbcode, reply_post, &config.thread.hash, &config.users))
                }
                @if let Some(text) = &post.text {
                    div."content bbcode" data-postid=(i(&post.id)) { (PreEscaped(bbcode.parse_profiled_opt(text, format!("post-{}",i(&post.id))))) }
                }
                div."postfooter mediumseparate" {
                    @if let Some(reply_link) = reply_chain_link {
                        details."repliesview aside" style="display:none" {
                            summary { "View conversation" }
                            iframe data-src=(reply_link){}
                        }
                    }
                    div."history" {
                        time."aside" datetime=(d(&post.createDate)) { (timeago_o(&post.createDate)) }
                        @if let Some(edit_user_id) = post.editUserId {
                            time."aside" datetime=(d(&post.editDate)) {
                                "Edited "(timeago_o(&post.editDate))" by "
                                @if let Some(edit_user) = users.get(&edit_user_id) {
                                    a."flatlink" target="_top" href=(layout_data.links.user(&edit_user)){ (&edit_user.username) }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn post_reply(
    layout_data: &MainLayoutData,
    bbcode: &mut BBCode,
    post: &Message,
    thash: &str,
    users: &HashMap<i64, User>,
) -> Markup {
    let user = user_or_default(users.get(&post.createUserId.unwrap_or(0)));
    html! {
        div."reply aside" {
            a."replylink" target="_top" href=(layout_data.links.forum_post_unsafe(post.id.unwrap_or_default(), thash)) { "Replying to:" }
            img src=(layout_data.links.image(&user.avatar, QueryImage::Cropped100 ));
            a."flatlink username" href=(layout_data.links.user(&user)) { (&user.username) }
            @if let Some(text) = &post.text {
                div."content bbcode postpreview" { (PreEscaped(bbcode.parse_profiled_opt(text, format!("reply-{}",i(&post.id))))) }
            }
        }
    }
}
