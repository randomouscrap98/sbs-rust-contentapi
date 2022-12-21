use std::collections::HashMap;

use super::*;
use contentapi::*;
use forum::*;
use submission::*;
use pagination::*;
use maud::*;


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
            link: format!("/forum/category/{}", if let Some(ref hash) = category.hash { hash } else { "" }),
            title: if let Some(ref name) = category.name { name.clone() } else { String::from("NOTFOUND") }
        }
    }
    pub fn from_thread(thread: &Content) -> Self {
        Self {
            link: format!("/forum/thread/{}", if let Some(ref hash) = thread.hash { hash } else { "" }),
            title: if let Some(ref name) = thread.name { name.clone() } else { String::from("NOTFOUND") }
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
            @if thread.neutral { (submission::pageicon(config, &thread.thread)) }
            @if thread.sticky { span{"📌"} }
            @if thread.locked { span{"🔒"} }
        }
    }
}


// ----------------------------
// *     BIG JUNK (THReAD)    *
// ----------------------------

// Unfortunately need this in here so the post knows how to render the iframe
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ThreadQuery {
    pub reply: Option<i64>,
    pub selected: Option<i64>
}

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
        //let mut res: Vec<i64> = values.iter()
        //    .filter(|(k,_v)| k.starts_with("re:"))
        //    .map(|(_k,v)| v.as_i64().unwrap_or_else(||0i64))
        //    .collect();
        //res.sort();
        //Some(res)
    }
    return None
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
                @if let Some(path) = &config.path {
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
                @let sequence = config.start_num.and_then(|s| Some(s + index as i32));
                (post_item(&context.layout_data, &mut context.bbcode, &config, post, sequence)) 
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

pub fn post_item(layout_data: &MainLayoutData, bbcode: &mut BBCode, config: &PostsConfig, post: &Message, sequence: Option<i32>) -> Markup// config: &LinkConfig, bbcode: &mut BBCode, post: &Message,  //thread: &Content, selected_post_id: Option<i64>, 
    //users: &HashMap<i64, User>, sequence: i32) -> Markup 
{
    let users = &config.users;
    let user = user_or_default(users.get(&post.createUserId.unwrap_or(0)));
    let mut class = String::from("post");
    if config.selected_post_id == post.id { class.push_str(" current") }
    let mut reply_chain_link: Option<String> = None;
    let mut reply_link: Option<String> = None;

    //Don't do the (not particularly expensive) calculation of all the reply links if the user didn't ask for it!
    if config.render_reply_link {
        if let Some(replies) = get_replydata(post) {
            //if res.len() > 0 {
                let query = ThreadQuery {
                    reply: Some(replies.top),//res.first().copied(),
                    selected: post.id
                };
                match serde_urlencoded::to_string(query) {
                    Ok(query) => {
                        reply_chain_link = Some(format!("{}/widget/thread?{}", &layout_data.config.http_root, query));
                        //reply_link = Some(forum_post_link(&layout_data.config, &post, &config.thread.thread))
                    },
                    Err(error) => println!("ERROR: couldn't encode thread query!: {}", error)
                }
            //}
        }
    }

    html! {
        div.(class) #{"post_"(i(&post.id))} {
            div."postleft" {
                img."avatar" src=(image_link(&layout_data.config, &user.avatar, 100, true)); 
            }
            div."postright" {
                div."postheader" {
                    a."flatlink username" href=(user_link(&layout_data.config, &user)) { (&user.username) } 
                    @if let Some(sequence) = sequence {
                        a."sequence" target="_top" title=(i(&post.id)) href=(forum_post_link(&layout_data.config, post, &config.thread.thread)){ "#" (sequence) } 
                    }
                }
                //@if let some(reply_link) = reply_link {
                //    a."reply" href=(reply_link) { ">>"(i()) }
                //}
                @if let Some(text) = &post.text {
                    div."content bbcode" { (PreEscaped(bbcode.parse_profiled_opt(text, format!("post-{}",i(&post.id))))) }
                }
                div."postfooter mediumseparate" {
                    //div."aside id" { (i(&post.id)) }
                    @if let Some(reply_link) = reply_chain_link {
                        details."replychain aside" {
                            summary { "View conversation" }
                            iframe src=(reply_link){}
                        }
                    }
                    div."history" {
                        time."aside" datetime=(d(&post.createDate)) { (timeago_o(&post.createDate)) } 
                        @if let Some(edit_user_id) = post.editUserId {
                            time."aside" datetime=(d(&post.editDate)) { 
                                "Edited "(timeago_o(&post.editDate))" by "
                                @if let Some(edit_user) = users.get(&edit_user_id) {
                                    a."flatlink" href=(user_link(&layout_data.config,&edit_user)){ (&edit_user.username) }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
