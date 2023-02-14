use std::collections::HashMap;

use contentapi::*;
use contentapi::forms::*;
use maud::*;

use crate::*;
use crate::forms::*;
use crate::render::*;
use crate::constants::*;
use crate::forum::*;
use crate::pagination::*;
use crate::render::submissions;


// ----------------------------
// *       BASIC JUNK         *
// ----------------------------

//To build the forum path at the top
pub struct ForumPathItem {
    pub link: String,
    pub title: String
}

impl ForumPathItem {
    pub fn from_category(category: &Content) -> Self {
        Self {
            link: format!("/forum/category/{}", opt_s!(category.hash)),
            title: String::from(opt_s!(category.name, "NOTFOUND"))
        }
    }
    pub fn from_thread(thread: &Content) -> Self {
        Self {
            link: format!("/forum/thread/{}", opt_s!(thread.hash)),
            title: String::from(opt_s!(thread.name, "NOTFOUND"))
        }
    }
    pub fn root() -> Self {
        Self {
            link: String::from("/forum"),
            title: String::from("Root")
        }
    }
}

pub fn forum_path(config: &LinkConfig, path: &Vec<ForumPathItem>) -> Markup {
    html!{
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
pub fn threadicon(config: &LinkConfig, thread: &ForumThread) -> Markup { //neutral: bool, sticky: bool, locked: bool) -> Markup {
    html! {
        div."threadicon smallseparate" {
            @if thread.neutral { (submissions::pageicon(config, &thread.thread)) }
            @if thread.sticky { span title="Pinned" {"📌"} }
            @if thread.locked { span title="Locked (No posting)" {"🔒"} }
            @if thread.private { span title="Private (Only participants can view)" {"🤫"} }
        }
    }
}


// ----------------------------
// *     BIG JUNK (THReAD)    *
// ----------------------------

/// Render a single post on a thread.
pub struct PostsConfig {
    /// The thread that holds all the posts to render
    pub thread: ForumThread,
    pub related: HashMap<i64,Message>,
    pub users: HashMap<i64,User>,
    /// The path to this thread; if not given, path not rendered. Thread must also be given
    pub path: Option<Vec<ForumPathItem>>,
    /// The pages to navigate posts; not displayed if not given
    pub pages: Option<Vec<PagelistItem>>,
    pub start_num: Option<i32>,
    pub selected_post_id: Option<i64>,

    pub render_header: bool,
    pub render_page: bool,
    pub render_reply_chain: bool,
    pub render_reply_link: bool,
    //pub render_sequence: bool
}

impl PostsConfig {
    pub fn thread_mode(thread: ForumThread, related: HashMap<i64,Message>, users: HashMap<i64,User>, 
        path: Vec<ForumPathItem>, pages: Vec<PagelistItem>, start: i32, selected_post_id: Option<i64>) -> Self
    {
        Self {
            thread,
            related,
            users,
            path: Some(path),
            pages: Some(pages),
            start_num: Some(start),
            selected_post_id,
            render_header: true,
            render_page: true,
            render_reply_chain: false,
            render_reply_link: true
        }
    }
    pub fn reply_mode(thread: ForumThread, related: HashMap<i64,Message>, users: HashMap<i64,User>, selected_post_id: Option<i64>) -> Self {
        Self {
            thread,
            related,
            users,
            path: None,
            pages: None,
            start_num: None,
            selected_post_id,
            render_header: false,
            render_page: false,
            render_reply_chain: true,
            render_reply_link: false
        }
    }
}

struct ReplyData {
    top: i64,
    direct: i64
}

/// Compute the flattened reply data for the given message
fn get_replydata(post: &Message) -> Option<ReplyData>
{
    if let Some(values) = &post.values {
        if let Some(top) = values.get("re-top").and_then(|v| v.as_i64()) {
            if let Some(direct) = values.get("re").and_then(|v| v.as_i64()) {
                return Some(ReplyData { top, direct })
            }
        }
    }
    return None
}

struct ReplyTree<'a> {
    id: i64,
    post: &'a Message,
    children: Vec<ReplyTree<'a>>
}

impl<'a> ReplyTree<'a> {
    fn new(message: &'a Message) -> Self {
        ReplyTree { 
            id: message.id.unwrap_or_else(||0), 
            post: message, 
            children: Vec::new() 
        }
    }
}

fn posts_to_replytree(posts: &Vec<Message>) -> Vec<ReplyTree> 
{
    let mut temp_tree : Vec<ReplyTree> =  Vec::new(); 
    'outer: for post in posts.iter() {
        if let Some(data) = get_replydata(post) {
            //Slow and I don't care
            for node in temp_tree.iter_mut() {
                if node.id == data.direct {
                    node.children.push(ReplyTree::new(post));
                    continue 'outer;
                }
            }
            println!("WARN: could not find place for message {}, reply to {}", i(&post.id), data.direct);
        }
        temp_tree.push(ReplyTree::new(post));
    }
    temp_tree
}

fn walk_post_tree(layout_data: &MainLayoutData, bbcode: &mut BBCode, config: &PostsConfig, tree: &ReplyTree, 
    sequence: Option<i32>, posts_left: &mut i32) -> Markup
{
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
pub fn render_posts(context: &mut PageContext, config: PostsConfig) -> Markup
{
    let thread = &config.thread;

    let is_pagetype = thread.thread.literalType.as_deref() == Some(SBSPageType::PROGRAM) ||
            thread.thread.literalType.as_deref() == Some(SBSPageType::RESOURCE);

    if is_pagetype {
        context.layout_data.override_nav_path = Some("/search");
    }
    else if thread.thread.literalType.as_deref() == Some(SBSPageType::DIRECTMESSAGE) {
        context.layout_data.override_nav_path = Some("/userhome");
    }

    let data = &context.layout_data;
    let bbcode = &mut context.bbcode;
    let mut post_count = config.thread.posts.len() as i32;

    let reply_tree: Vec<ReplyTree> = if config.render_reply_chain 
    {
        posts_to_replytree(&thread.posts)
    }
    else {
        // no reply chain is just a simple list of whatever
        config.thread.posts.iter().map(|m| ReplyTree::new(m)).collect() 
    };

    html!{
        (data.links.style("/forpage/forum.css"))
        (data.links.script("/forpage/forum.js"))
        @if config.render_header {
            section {
                h1 { (opt_s!(thread.thread.name)) }
                @if let Some(path) = &config.path {
                    (forum_path(&data.links, &path))
                }
                div."foruminfo smallseparate aside" {
                    (threadicon(&data.links, &thread))
                    span {
                        @if let Some(user) = config.users.get(&thread.thread.createUserId.unwrap_or(0)) {
                            a."flatlink" target="_top" href=(data.links.user(user)){ (user.username) }
                        }
                    }
                    span {
                        b { "Created: " }
                        time datetime=(d(&thread.thread.createDate)) { (timeago_o(&thread.thread.createDate)) }
                    }
                }
            }
        }
        @if config.render_page && is_pagetype {
            (render_page(&data, bbcode, &thread))
        }
        //it says "thread-top" because it is: it's the beginning of the section that displays posts. After the 
        //for loop, it then displays pages, which is on the bottom of the thread, so it might seem confusing.
        //maybe the id should be changed to an anchor, idr how to do that.
        section #"thread-top" data-selected=[config.selected_post_id] {
            @for (index,tree) in reply_tree.iter().enumerate() {
                @let sequence = config.start_num.and_then(|s| Some(s + index as i32));
                (walk_post_tree(&context.layout_data, &mut context.bbcode, &config, tree, sequence, &mut post_count))
            }
            @if let Some(pages) = config.pages {
                div."smallseparate pagelist" {
                    @for page in pages {
                        a."current"[page.current] target="_top" href={(data.links.forum_thread(&thread.thread))"?page="(page.page)"#thread-top"} { (page.text) }
                    }
                }
            }
            //Only display the thread controls if it's NOT a regular page
            @if let Some(ref user) = context.layout_data.user {
                //TODO: again, reusing pagelist may be inappropriate. IDK
                div."smallseparate pagelist" {
                    @if can_create_post(user, &thread.thread) {
                        a."coolbutton" #"createpost" href=(data.links.forum_post_editor_new(&thread.thread, None)) { "New post" }
                    }
                    @if !is_pagetype {
                        @if can_edit_thread(user, &thread.thread) {
                            a."coolbutton" #"editthread" href=(data.links.forum_thread_editor_edit(&thread.thread)) { "Edit thread" }
                        }
                        @if can_delete_thread(user, &thread.thread) {
                            form."nospacing" #"deletethread" method="POST" action=(data.links.forum_thread_delete(&thread.thread)) {
                                input."coolbutton" data-confirmdelete=(format!("thread '{}'", opt_s!(&thread.thread.name))) type="submit" value="Delete thread";
                            }
                        }
                    }
                }
            }
        }
    }
}

fn images_to_attr(config: &LinkConfig, images: &Vec<serde_json::Value>) -> String 
{
    serde_json::to_string(
        &images.iter().map(|i| {
            match i.as_str() {
                Some(string) => config.image_default(string),
                None => {
                    println!("ERROR: IMAGE HASH NOT STRING: {}", i);
                    String::new()
                }
            }
        }).collect::<Vec<String>>()
    ).unwrap_or_else(|err| {
        println!("ERROR: COULD NOT SERIALIZE PAGE IMAGES: {}", err);
        String::new()
    })
}

/// Render the page data, such as text and infoboxes, on standard pages. True forum threads don't have main
/// content like that, so this is only called on programs, resources, etc
pub fn render_page(data: &MainLayoutData, bbcode: &mut BBCode, thread: &ForumThread) -> Markup 
{
    let values = match &thread.thread.values { Some(values) => values.clone(), None => HashMap::new() };
    html!{
        section {
            //First check is if it's a program, then we float this box to the right
            @if thread.thread.literalType.as_deref() == Some(SBSPageType::PROGRAM) {
                div."programinfo" {
                    @if let Some(images) = values.get(SBSValue::IMAGES).and_then(|k| k.as_array()) {
                        div."gallery" #"page_gallery" /*data-index="0"*/ data-images=(images_to_attr(&data.links, &images)) {
                            //we now have the images: we just need the first one (it's a hash?)
                            @if let Some(image) = images.get(0).and_then(|i| i.as_str()) {
                                img src=(data.links.image_default(image));
                            }
                        }
                    }
                    div."extras mediumseparate" {
                        @if let Some(key) = values.get(SBSValue::DOWNLOADKEY).and_then(|k| k.as_str()) {
                            span."smallseparate" {
                                b { "Download:" }
                                span."key" { (key) }
                                (threadicon(&data.links, &thread))
                            }
                        }
                        @if let Some(version) = values.get(SBSValue::VERSION).and_then(|k| k.as_str()) {
                            span."smallseparate" {
                                b { "Version:" }
                                span."version" { (version) }
                            }
                        }
                        @if let Some(size) = values.get(SBSValue::SIZE).and_then(|k| k.as_str()) {
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

//WAS consuming bbcode, now i'm not sure. leaving for now
pub fn post_item(layout_data: &MainLayoutData, bbcode: &mut BBCode, config: &PostsConfig, post: &Message, 
    sequence: Option<i32>) -> Markup
{
    let users = &config.users;
    let user = user_or_default(users.get(&post.createUserId.unwrap_or(0)));
    let mut class = String::from("post");
    if config.selected_post_id == post.id { class.push_str(" current") }
    let mut reply_chain_link: Option<String> = None;
    let mut reply_post : Option<&Message> = None;

    if let Some(replies) = get_replydata(post) {
        reply_post = config.related.get(&replies.direct);
        if reply_post.is_none() {
            println!("ERROR: couldn't find related post {}!", replies.direct)
        }
        if config.render_reply_link {
            let query = ThreadQuery {
                reply: Some(replies.top),
                selected: post.id
            };
            match serde_urlencoded::to_string(query) {
                Ok(query) => {
                    reply_chain_link = Some(format!("{}/widget/thread?{}", &layout_data.links.http_root, query)); //, forum_post_hash(post)));
                },
                Err(error) => println!("ERROR: couldn't encode thread query!: {}", error)
            }
        }
    }

    html! {
        div.(class) #{"post_"(i(&post.id))} {
            div."postleft" {
                img."avatar" src=(layout_data.links.image(&user.avatar, &QueryImage::avatar(100))); 
                @if config.thread.private {
                    div."private" { "PRIVATE" }
                }
            }
            div."postright" {
                div."postheader" {
                    a."flatlink username" target="_top" href=(layout_data.links.user(&user)) { (&user.username) } 
                    @if let Some(sequence) = sequence {
                        a."sequence" target="_top" title=(i(&post.id)) href=(layout_data.links.forum_post(post, &config.thread.thread)){ "#" (sequence) } 
                    }
                }
                @if let Some(reply_post) = reply_post {
                    //TODO: can't decide between consuming or not. spoilers are the important bit
                    (post_reply(layout_data, bbcode, reply_post, &config.thread.thread, &config.users))
                }
                @if let Some(text) = &post.text {
                    div."content bbcode" { (PreEscaped(bbcode.parse_profiled_opt(text, format!("post-{}",i(&post.id))))) }
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

pub fn post_reply(layout_data: &MainLayoutData, bbcode: &mut BBCode, post: &Message, thread: &Content, users: &HashMap<i64, User>) -> Markup
{
    let user = user_or_default(users.get(&post.createUserId.unwrap_or(0)));
    html! {
        div."reply aside" {
            a."replylink" target="_top" href=(layout_data.links.forum_post(post, thread)) { "Replying to:" }
            img src=(layout_data.links.image(&user.avatar, &QueryImage::avatar(50))); 
            a."flatlink username" href=(layout_data.links.user(&user)) { (&user.username) } 
            @if let Some(text) = &post.text {
                //Ignoring graphemes for now, sorry. In NEARLY all cases, 200 bytes should be enough to fill 
                //a line, unless you're being ridiculous
                //@let text = if text.len() > 200 { &text[0..200] } else { &text };
                div."content bbcode postpreview" { (PreEscaped(bbcode.parse_profiled_opt(text, format!("reply-{}",i(&post.id))))) }
            }
        }
    }
}

//Eventually may expand this
pub fn post_textbox(id: Option<&str>, name: Option<&str>, value: Option<&str>) -> Markup
{
    html! {
        textarea id=(opt_s!(id)) type="text" name=(opt_s!(name)) required 
        placeholder=r##"[b]bold[/b], [i]italic[/i], 
[u]underline[/u], [s]strikethrough[/s], 
[spoiler=text]hidden[/spoiler], [quote=user]text[/quote]
        "## { (opt_s!(value)) }
    }
}