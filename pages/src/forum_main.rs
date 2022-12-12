use std::collections::HashMap;

use contentapi::User;

use super::*;

//Not sure if we need values, but I NEED permissions to know if the thread is locked
static THREADFIELDS : &str = "id,name,lastCommentId,literalType,hash,parentId,commentCount,createDate,createUserId,values,permissions";
//Need values to know the stickies
static CATEGORYFIELDS: &str = "id,hash,name,description,literalType,values";
static THREADKEY: &str = "thread";
static CATEGORYKEY: &str = "category";
static PREMESSAGEKEY: &str = "premessage";
static PREMESSAGEINDEXKEY: &str = "premessage_index";

struct Keygen();

impl Keygen {
    fn threadcount(id: i64) -> String { format!("threadcount_{id}") }
    fn threads(id: i64) -> String { format!("threads_{id}") }
    fn stickythreads(id: i64) -> String { format!("stickythreads_{id}") }
    fn stickies(id: i64) -> String { format!("stickies_{id}")}
}

#[derive(Clone, Debug)]
pub struct ForumThread {
    thread: Content,
    sticky: bool,
    locked: bool,
    neutral: bool, //Used by the frontend
    posts: Vec<Message>
}

impl ForumThread {
    fn from_content(thread: Content, messages_raw: &Vec<Message>, stickies: &Vec<i64>) -> Result<Self, Error> {
        let thread_id = thread.id;
        let permissions = match thread.permissions {
            Some(ref p) => Ok(p),
            None => Err(Error::Other(String::from("Thread didn't have permissions in resultset!")))
        }?;
        //"get" luckily already gets the thing as a reference
        let global_perms = permissions.get("0").ok_or(Error::Other(String::from("Thread didn't have global permissions!")))?;
        let locked = !global_perms.contains('C'); //Right... the order matters. need to finish using it before you give up thread
        let sticky = stickies.contains(&thread_id.unwrap_or(0));
        Ok(ForumThread { 
            locked, sticky, thread,
            neutral: !locked && !sticky,
            posts: messages_raw.iter().filter(|m| m.contentId == thread_id).map(|m| m.clone()).collect()
        })
    }
}

//Structs JUST for building data for the forum templates (so no need to be public)
#[derive(Clone, Debug)]
pub struct ForumCategory {
    category: Content,
    threads: Vec<ForumThread>,
    stickies: Vec<ForumThread>,
    threads_count: i32,
    users: HashMap<String, User>
}


pub fn render(data: MainLayoutData, categories: Vec<ForumCategory>) -> String {
    layout(&data, html!{
        (style(&data.config, "/forum.css"))
        section { h1 { "Forum Topics" } }
        section {
            @for (index, category_container) in categories.iter().enumerate() {
                @let category = &category_container.category;
                div."category" {
                    div."categoryinfo" {
                        h1 { a."flatlink" href=(forum_category_link(&data.config, &category)) {(s(&category.name))} }
                        p."aside" {(s(&category.description))}
                    }
                    div."foruminfo aside mediumseparate" {
                        div { b{"Threads: "} (category_container.threads_count) }
                        @if let Some(thread) = category_container.threads.get(0) {
                            div {
                                @if let Some(post) = thread.posts.get(0) {
                                    b { time datetime=(d(&post.createDate)) { (timeago_o(&post.createDate)) } }
                                    a."flatlink" href=(forum_post_link(&data.config, post, &thread.thread)) { (s(&thread.thread.name)) } 
                                }
                            }
                        }
                    }
                }
                @if index < categories.len() - 1 {
                    hr;
                }
            }
        }
    }).into_string()
}
