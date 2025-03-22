use chrono::{DateTime, Utc};
use core::fmt::Debug;
use maud::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::common::contentapi::*;
use super::common::render::*;
use super::common::*;
use super::context::*;
use crate::layout::*;
use crate::links::*;
use crate::response::*;
use crate::*;

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ThreadQuery {
    pub reply: Option<i64>,
    pub selected: Option<i64>,
}

static COMMONMESSAGE: &str = "m.deleted = 0 AND m.module IS NULL AND m.contentId IN (SELECT contentId FROM content_permissions WHERE read=1 AND userId=0)";
static MSGVALUESELECT: &str = "SELECT `key`,`value` FROM message_values WHERE messageId=?";

fn get_thread_by_hash(ctx: &PageContext, hash: &str) -> Result<Option<ForumThread2>, Error> {
    let (mut query, mut params) = forum_theads_base_query();
    query.push_str(" AND t.hash = ?");
    params.push(Box::new(hash));
    Ok(gather_forum_threads(&query, ctx, box_to_ref!(params))?.pop())
}

fn get_forum_category_by_id(ctx: &PageContext, id: i64) -> Result<Option<ForumCategory2>, Error> {
    let (mut query, mut params) = forum_categories_base_query();
    query.push_str(" AND c.id = ?");
    params.push(Box::new(id));
    let mut stmt = ctx.dbcon.prepare(&query)?;
    let mut result = gather_forum_categories((&mut stmt, &query), box_to_ref!(params))?;
    Ok(result.pop())
}

#[derive(Clone, Debug)]
struct ForumPost {
    pub id: i64,
    // pub content_id: i64,
    pub create_user_id: i64,
    pub create_date: DateTime<Utc>,
    pub text: String,
    pub edit_date: Option<DateTime<Utc>>,
    pub edit_user_id: Option<i64>,
    pub values: HashMap<String, String>,
}

fn gather_forum_posts(
    query: &str,
    ctx: &PageContext,
    params: &[&dyn rusqlite::ToSql],
) -> Result<Vec<ForumPost>, Error> {
    let mut stmt = ctx.dbcon.prepare(query)?;
    let mut vstmt = ctx.dbcon.prepare(MSGVALUESELECT)?;
    easy_query((&mut stmt, query), params, |row| {
        let id = row.get(0)?;
        Ok(ForumPost {
            id,
            //content_id: row.get(1)?,
            create_user_id: row.get(2)?,
            create_date: row.get(3)?,
            text: row.get(4)?,
            edit_date: row.get(5)?,
            edit_user_id: row.get(6)?,
            values: gather_values(&mut vstmt, id)?,
        })
    })
}

fn map_forumposts(posts: Vec<ForumPost>) -> HashMap<i64, ForumPost> {
    posts
        .into_iter()
        .map(|u| (u.id, u))
        .collect::<HashMap<i64, ForumPost>>()
}

fn add_uids_from_posts(messages: &Vec<ForumPost>, existing: &mut Vec<i64>) {
    for message in messages {
        existing.push(message.create_user_id);
        if let Some(edit_uid) = message.edit_user_id {
            existing.push(edit_uid);
        }
    }
}

fn forum_posts_base_query() -> (String, Vec<Box<dyn rusqlite::ToSql>>) {
    let query = format!(
        "SELECT {} FROM messages m WHERE {}",
        "m.id,m.contentId,m.createUserId,m.createDate,m.text,m.editDate,m.editUserId",
        COMMONMESSAGE,
    );
    let params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![]; //Box::new(ContentType::PAGE)];
    (query, params)
}

fn get_forumposts_basic(
    ctx: &PageContext,
    content_id: i64,
    ids: Option<Vec<i64>>,
    limit: QueryLimit,
) -> Result<Vec<ForumPost>, Error> {
    let (mut query, mut params) = forum_posts_base_query();
    query.push_str(" AND m.contentId = ?");
    params.push(Box::new(content_id));
    if let Some(ids) = ids {
        query.push_str(" AND m.id IN (");
        query.push_str(&params_list(ids.len()));
        query.push_str(")");
        for id in ids {
            params.push(Box::new(id));
        }
    }
    limit.mod_query(&mut query, &mut params);
    gather_forum_posts(&query, ctx, box_to_ref!(params))
}

fn get_forumposts_replies(ctx: &PageContext, root_post_id: i64) -> Result<Vec<ForumPost>, Error> {
    let (mut query, mut params) = forum_posts_base_query();
    query.push_str(
        " AND (m.id = ? OR m.id IN (SELECT messageId FROM message_values WHERE `key`=? AND `value`=?))",
    );
    params.push(Box::new(root_post_id));
    params.push(Box::new("re-top"));
    params.push(Box::new(format!("{}", root_post_id)));
    gather_forum_posts(&query, ctx, box_to_ref!(params))
}

fn get_count_before_forumpost(ctx: &PageContext, id: i64) -> Result<i32, Error> {
    let query = format!(
        "SELECT COUNT(*) FROM messages m WHERE {} AND m.id < ? AND m.contentId = (SELECT contentId FROM messages WHERE id = ?)",
        COMMONMESSAGE
    );
    let mut stmt = ctx.dbcon.prepare(&query)?;
    let mut result = easy_query((&mut stmt, &query), rusqlite::params![id, id], |row| {
        row.get::<usize, i32>(0)
    })?;
    result.pop().ok_or(Error::NotFound(String::from(
        "PROGRAM ERROR: No results on count(*)??",
    )))
}

/// Render a single post on a thread.
struct PostsConfig {
    /// The thread that holds all the posts to render
    thread: ForumThread2,
    categories: Vec<SubmissionCategory>,
    posts: Vec<ForumPost>,
    related: HashMap<i64, ForumPost>,
    users: HashMap<i64, User2>,
    /// The path to this thread; if not given, path not rendered. Thread must also be given
    path: Option<Vec<ForumPathItem>>,
    /// The pages to navigate posts; not displayed if not given
    pages: Option<Vec<PagelistItem>>,
    start_num: Option<i32>,
    selected_post_id: Option<i64>,
    render_header: bool,
    render_page: bool,
    render_reply_chain: bool,
    render_reply_link: bool,
}

impl PostsConfig {
    pub fn thread_mode(
        thread: ForumThread2,
        categories: Vec<SubmissionCategory>,
        posts: Vec<ForumPost>,
        related: HashMap<i64, ForumPost>,
        users: HashMap<i64, User2>,
        path: Vec<ForumPathItem>,
        pages: Vec<PagelistItem>,
        start: i32,
        selected_post_id: Option<i64>,
    ) -> Self {
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
        }
    }
    pub fn reply_mode(
        thread: ForumThread2,
        posts: Vec<ForumPost>,
        related: HashMap<i64, ForumPost>,
        users: HashMap<i64, User2>,
        selected_post_id: Option<i64>,
    ) -> Self {
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
        }
    }
}

pub const SHORTDESCRIPTION: usize = 200;

fn short_post(message: &ForumPost) -> String {
    message
        .text
        .chars()
        .take(SHORTDESCRIPTION)
        .collect::<String>()
}

fn short_description2(thread: &ForumThread2) -> String {
    if !thread.description.is_empty() {
        return thread.description.clone();
    }
    if thread.literal_type != SBSPageType::FORUMTHREAD {
        //Get some short portion of the body, even if it's bad? We'll fix bbcode stuff later
        if !thread.text.is_empty() {
            return thread
                .text
                .chars()
                .take(SHORTDESCRIPTION)
                .collect::<String>();
        }
    }
    return String::from("");
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

pub fn get_thumbnail_hash2(content: &ForumThread2) -> Option<String> {
    if let Some(ref images) = get_value_safe!(&content.values, SBSValue::IMAGES, Vec<String>) {
        //values.get(SBSValue::IMAGES).and_then(|k| k.as_array()) {
        if let Some(image) = images.get(0) {
            return Some(image.clone());
        }
    }

    None
}

/// Render the page data, such as text and infoboxes, on standard pages. True forum threads don't have main
/// content like that, so this is only called on programs, resources, etc
pub fn render_page(
    data: &MainLayoutData,
    bbcode: &mut BBCode,
    thread: &ForumThread2,
    categories: &Vec<SubmissionCategory>,
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

pub fn render_textcontent(
    text: &str,
    values: &HashMap<String, String>,
    bbcode: &mut BBCode,
) -> Markup {
    let mut markup: &str = MARKUPBBCODE;
    if let Some(mkup) = get_value_safe!(values, SBSValue::MARKUP, &str) {
        markup = mkup;
    }
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

#[derive(Clone, Debug)]
pub struct ReplyData {
    pub top: i64,
    pub direct: i64,
}

/// Compute the flattened reply data for the given message
fn get_replydata(post: &ForumPost) -> Option<ReplyData> {
    if let Some(top) = get_value_safe!(post.values, "re-top", i64) {
        if let Some(direct) = get_value_safe!(post.values, "re", i64) {
            return Some(ReplyData { top, direct });
        }
    }
    return None;
}

#[derive(Clone, Debug)]
struct ReplyTree<'a> {
    pub id: i64,
    pub post: &'a ForumPost,
    pub children: Vec<ReplyTree<'a>>,
}

impl<'a> ReplyTree<'a> {
    pub fn new(message: &'a ForumPost) -> Self {
        ReplyTree {
            id: message.id,
            post: message,
            children: Vec::new(),
        }
    }

    /// Insert the given post as a node in this tree. Modifies the tree, and returns the node (if it was inserted)
    pub fn insert_post(&mut self, post: &'a ForumPost, data: &ReplyData) -> Option<&ReplyTree> {
        //If this is the node to insert into, return ourselves
        if self.id == data.direct {
            self.children.push(ReplyTree::new(post));
            return self.children.last();
        } else {
            //Otherwise, look through all the children. This is depth first recursion... whatever I guess.
            for node in self.children.iter_mut() {
                let result = node.insert_post(post, data);
                if result.is_some() {
                    return result;
                }
            }
        }

        //If we make it here, it was never found
        return None;
    }
}

/// Convert a list of posts into a tree. ASSUMES THE FIRST POST IS THE ROOT!!
fn posts_to_replytree(posts: &Vec<ForumPost>) -> Vec<ReplyTree> {
    if posts.len() == 0 {
        return Vec::new();
    }

    let mut root = ReplyTree::new(&posts[0]);

    for post in posts.iter().skip(1) {
        if let Some(data) = get_replydata(post) {
            if root.insert_post(post, &data).is_none() {
                println!(
                    "WARN: could not find place for message {}, reply to {}",
                    post.id, data.direct
                );
            }
        }
    }

    //println!("{:#?}", root);
    vec![root]
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
fn render_mainview(context: &mut PageContext, config: PostsConfig) -> Markup {
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
                                a."flatlink" target="_top" href=(data.links.user_unsafe(&user.username)){ (user.username) }
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

//WAS consuming bbcode, now i'm not sure. leaving for now
fn post_item(
    layout_data: &MainLayoutData,
    bbcode: &mut BBCode,
    config: &PostsConfig,
    post: &ForumPost,
    sequence: Option<i32>,
) -> Markup {
    let users = &config.users;
    let user = user_or_default2(users.get(&post.create_user_id));
    let mut class = String::from("post");
    if config.selected_post_id == Some(post.id) {
        class.push_str(" current")
    }
    let mut reply_chain_link: Option<String> = None;
    let mut reply_post: Option<&ForumPost> = None;

    if let Some(replies) = get_replydata(post) {
        reply_post = config.related.get(&replies.direct);
        if reply_post.is_none() {
            println!("ERROR: couldn't find related post {}!", replies.direct)
        }
        if config.render_reply_link {
            let query = ThreadQuery {
                reply: Some(replies.top),
                selected: Some(post.id),
            };
            match serde_urlencoded::to_string(query) {
                Ok(query) => {
                    reply_chain_link = Some(format!(
                        "{}/widget/thread?{}",
                        &layout_data.links.http_root, query
                    ));
                }
                Err(error) => println!("ERROR: couldn't encode thread query!: {}", error),
            }
        }
    }

    html! {
        div.(class) #{"post_"(post.id)} {
            div."postleft" {
                img."avatar" src=(layout_data.links.image(&user.avatar, QueryImage::Cropped100 ));
            }
            div."postright" {
                div."postheader" {
                    a."flatlink username" target="_top" href=(layout_data.links.user_unsafe(&user.username)) { (&user.username) }
                    @if let Some(sequence) = sequence {
                        a."sequence" target="_top" title=(post.id) href=(layout_data.links.forum_post_unsafe(post.id, &config.thread.hash)){ "#" (sequence) }
                    }
                }
                @if let Some(reply_post) = reply_post {
                    //TODO: can't decide between consuming or not. spoilers are the important bit
                    (post_reply(layout_data, bbcode, reply_post, &config.thread.hash, &config.users))
                }
                div."content bbcode" data-postid=(post.id) { (PreEscaped(bbcode.parse(&post.text))) }
                div."postfooter mediumseparate" {
                    @if let Some(reply_link) = reply_chain_link {
                        details."repliesview aside" style="display:none" {
                            summary { "View conversation" }
                            iframe data-src=(reply_link){}
                        }
                    }
                    div."history" {
                        time."aside" datetime=(dd(&post.create_date)) { (timeago(&post.create_date)) }
                        @if let Some(edit_user_id) = post.edit_user_id {
                            time."aside" datetime=(d(&post.edit_date)) {
                                "Edited "(timeago_o(&post.edit_date))" by "
                                @if let Some(edit_user) = users.get(&edit_user_id) {
                                    a."flatlink" target="_top" href=(layout_data.links.user_unsafe(&edit_user.username)){ (&edit_user.username) }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn post_reply(
    layout_data: &MainLayoutData,
    bbcode: &mut BBCode,
    post: &ForumPost,
    thash: &str,
    users: &HashMap<i64, User2>,
) -> Markup {
    let user = user_or_default2(users.get(&post.create_user_id));
    html! {
        div."reply aside" {
            a."replylink" target="_top" href=(layout_data.links.forum_post_unsafe(post.id, thash)) { "Replying to:" }
            img src=(layout_data.links.image(&user.avatar, QueryImage::Cropped100 ));
            a."flatlink username" href=(layout_data.links.user_unsafe(&user.username)) { (&user.username) }
            div."content bbcode postpreview" { (PreEscaped(bbcode.parse(&post.text))) }
        }
    }
}

fn render(mut context: PageContext, config: PostsConfig) -> String {
    let mut meta = LayoutMeta {
        title: format!("SBS ⦁ {}", config.thread.name),
        description: short_description2(&config.thread),
        image: get_thumbnail_hash2(&config.thread)
            .and_then(|h| Some(context.layout_data.links.image(&h, QueryImage::Scaled300))),
        canonical: Some(
            context
                .layout_data
                .links
                .forum_thread_unsafe(&config.thread.hash),
        ),
    };

    //If the literal type is a forumthread, we generally want to pull text from posts
    if config.thread.literal_type == SBSPageType::FORUMTHREAD {
        if let Some(selected_id) = config.selected_post_id {
            if let Some(post) = config.posts.iter().find(|p| p.id == selected_id) {
                meta.description = short_post(post);
                meta.canonical = Some(
                    context
                        .layout_data
                        .links
                        .forum_post_unsafe(post.id, &config.thread.hash),
                );
            }
        } else if let Some(start) = config.start_num {
            if start == 1 {
                //We KNOW this is the first page, so we can actually give the post as the description!
                if let Some(post) = config.posts.get(0) {
                    meta.description = short_post(post);
                }
            }
        }
    }

    let main_page = render_mainview(&mut context, config);
    layout_with_meta(&context.layout_data, meta, main_page).into_string()
}

fn get_related_posts(
    context: &PageContext,
    posts_raw: &Vec<ForumPost>,
    thread_id: i64,
) -> Result<Vec<ForumPost>, Error> {
    let mut related_ids: Vec<i64> = Vec::new();
    for message in posts_raw {
        if let Some(data) = get_replydata(message) {
            related_ids.push(data.direct);
        }
    }
    get_forumposts_basic(context, thread_id, Some(related_ids), QueryLimit::default())
}

async fn render_thread(
    context: PageContext,
    hash: &str,
    post_id: Option<i64>,
    per_page: i32,
    page: Option<i32>,
) -> Result<Response, Error> {
    let mut page = page.unwrap_or(1) - 1; //we assume 1-based pages

    let thread = get_thread_by_hash(&context, hash)?
        .ok_or(Error::NotFound(String::from("Could not find thread!")))?;
    let category = get_forum_category_by_id(&context, thread.parent_id)?
        .ok_or(Error::NotFound(String::from("Could not find category!")))?;
    let thread_subcat_ids = get_tagged_categories2(&thread.values);
    let subcategories = get_submission_categories(&context, Some(thread_subcat_ids))?;

    if let Some(post_id) = post_id {
        let precount = get_count_before_forumpost(&context, post_id)?;
        //The index is the special count. This means we change the page given. If page wasn't already 0, we warn
        if page != 0 {
            println!(
                "Page was nonzero ({}) while there was a message index ({})",
                page, precount
            );
        }
        page = precount / per_page;
    }

    //Also I need some fields to exist.
    let thread_id = thread.id;
    let thread_create_uid = thread.create_user_id;
    let comment_count = thread.posts_count;

    let sequence_start = page * per_page;

    let messages_raw = get_forumposts_basic(
        &context,
        thread_id,
        None,
        QueryLimit {
            limit: Some(per_page),
            skip: Some(sequence_start),
        },
    )?;
    let related_raw = get_related_posts(&context, &messages_raw, thread_id)?;
    let mut user_ids: Vec<i64> = vec![thread_create_uid];
    add_uids_from_posts(&messages_raw, &mut user_ids);
    add_uids_from_posts(&related_raw, &mut user_ids);
    let users_raw = get_users(&context, user_ids)?;

    //Construct before borrowing
    let path = vec![
        ForumPathItem::root(),
        ForumPathItem::from_category2(&category),
        ForumPathItem::from_thread2(&thread),
    ];
    let post_config = PostsConfig::thread_mode(
        thread,
        subcategories,
        messages_raw,
        map_forumposts(related_raw),
        map_users2(users_raw),
        path,
        get_pagelist(comment_count as i32, per_page, page),
        1 + per_page * page,
        post_id,
        //selected_post.and_then(|m| m.id),
    );
    Ok(Response::Render(render(context, post_config)))
}

/// The normal endpoint for listing a thread
pub async fn get_hash_render(
    context: PageContext,
    hash: String,
    per_page: i32,
    page: Option<i32>,
) -> Result<Response, Error> {
    let hurgh = hash.clone();
    render_thread(context, &hurgh, None, per_page, page).await
}

/// The normal endpoint for pinpointing a post
pub async fn get_hash_postid_render(
    context: PageContext,
    hash: String,
    post_id: i64,
    per_page: i32,
) -> Result<Response, Error> {
    let hurgh = hash.clone();
    render_thread(context, &hurgh, Some(post_id), per_page, None).await
}

// -------------------------------------
//          Widget thread
// -------------------------------------

/// Rendering for the actual widget. The
fn render_widget(context: &mut PageContext, config: PostsConfig) -> String {
    let posts = render_mainview(context, config);
    basic_skeleton(
        &context.layout_data,
        html! {
            title { "SmileBASIC Source Thread Widget" }
            meta name="description" content="Simple view into a thread";
            (context.layout_data.links.style("/forpage/forum.css"))
            style { r#"
            body { 
                /* This shrinks the WHOLE page! */
                font-size: 0.85rem; 
                padding: var(--space_medium);
            }
            @media screen and (max-width: 30em) {
                font-size: 0.75rem; 
            }
        "# }
        },
        html! {
            (posts)
        },
    )
    .into_string()
}

fn get_thread_by_msgid(ctx: &PageContext, msgid: i64) -> Result<Option<ForumThread2>, Error> {
    let (mut query, mut params) = forum_theads_base_query();
    query.push_str(" AND t.id = (SELECT m.contentId FROM messages m WHERE m.id = ?)");
    params.push(Box::new(msgid));
    Ok(gather_forum_threads(&query, ctx, box_to_ref!(params))?.pop())
}

pub async fn get_render_widget(
    mut context: PageContext,
    query: ThreadQuery,
) -> Result<Response, Error> {
    if let Some(post_id) = query.reply {
        let thread = get_thread_by_msgid(&context, post_id)?
            .ok_or(Error::NotFound(String::from("Could not find thread!")))?;

        let messages_raw = get_forumposts_replies(&context, post_id)?;
        let related_raw = get_related_posts(&context, &messages_raw, thread.id)?;
        // println!("RELATED POSTS: {}", related_raw.len());
        let mut user_ids: Vec<i64> = Vec::new();
        add_uids_from_posts(&messages_raw, &mut user_ids);
        add_uids_from_posts(&related_raw, &mut user_ids);
        let users_raw = get_users(&context, user_ids)?;

        Ok(Response::Render(render_widget(
            &mut context,
            PostsConfig::reply_mode(
                thread,
                messages_raw,
                map_forumposts(related_raw),
                map_users2(users_raw),
                query.selected,
            ),
        )))
    } else {
        Err(Error::Other(String::from(
            "No data provided; this widget requires at least 'reply'",
        )))
    }
}
