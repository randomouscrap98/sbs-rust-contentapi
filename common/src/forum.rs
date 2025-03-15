use std::collections::HashMap;

use super::*;
use crate::capi::SubmissionCategory;
use crate::constants::*;
use crate::response::*;

use contentapi::*;

//Not sure if we need values, but I NEED permissions to know if the thread is locked
pub static THREADFIELDS : &str = "id,name,lastCommentId,literalType,contentType,hash,parentId,commentCount,createDate,createUserId,values,permissions,lastRevisionId,lastActionDate";
//Need values to know the stickies
// pub static CATEGORYFIELDS: &str = "id,hash,name,description,literalType,contentType";

//Note: these are keys for the REQUESTS, not anything else!
pub static THREADKEY: &str = "thread";
//pub static CATEGORYKEY: &str = "category";
pub static PREMESSAGEKEY: &str = "premessage";
pub static PREMESSAGEINDEXKEY: &str = "premessage_index";

// ---------------------------------------------
//  DIRECT CONNECT NEW
// ---------------------------------------------

#[derive(Clone, Debug)]
pub struct ForumThread {
    pub thread: Content,
    pub id: i64,
    pub posts: Vec<Message>,
    pub categories: Option<Vec<SubmissionCategory>>,
}

impl ForumThread {
    pub fn from_content(thread: Content, messages_raw: &Vec<Message>) -> Result<Self, Error> {
        let thread_id = thread.id.unwrap_or(0);
        //"get" luckily already gets the thing as a reference
        Ok(ForumThread {
            thread,
            id: thread_id,
            posts: messages_raw
                .iter()
                .filter(|m| m.contentId == Some(thread_id))
                .map(|m| m.clone())
                .collect(),
            categories: None,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ReplyData {
    pub top: i64,
    pub direct: i64,
}

impl ReplyData {
    pub fn write_to_values(&self, values: &mut HashMap<String, serde_json::Value>) {
        values.insert(String::from("re-top"), self.top.into());
        values.insert(String::from("re"), self.direct.into());
    }
}

/// Compute the flattened reply data for the given message
pub fn get_replydata(post: &Message) -> Option<ReplyData> {
    if let Some(values) = &post.values {
        if let Some(top) = values.get("re-top").and_then(|v| v.as_i64()) {
            if let Some(direct) = values.get("re").and_then(|v| v.as_i64()) {
                return Some(ReplyData { top, direct });
            }
        }
    }
    return None;
}

/// Given a post, regenerate the new reply data that would properly point to this post
pub fn get_new_replydata(post: &Message) -> ReplyData {
    let id = post.id.unwrap_or_default();

    let mut reply_data = ReplyData {
        top: id,
        direct: id,
    };

    //Oh but if we can parse existing reply data off the message, that one's top becomes our top too
    if let Some(existing) = get_replydata(post) {
        reply_data.top = existing.top;
    }

    reply_data
}

#[derive(Clone, Debug)]
pub struct ReplyTree<'a> {
    pub id: i64,
    pub post: &'a Message,
    pub children: Vec<ReplyTree<'a>>,
}

impl<'a> ReplyTree<'a> {
    pub fn new(message: &'a Message) -> Self {
        ReplyTree {
            id: message.id.unwrap_or_else(|| 0),
            post: message,
            children: Vec::new(),
        }
    }

    /// Insert the given post as a node in this tree. Modifies the tree, and returns the node (if it was inserted)
    pub fn insert_post(&mut self, post: &'a Message, data: &ReplyData) -> Option<&ReplyTree> {
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
pub fn posts_to_replytree(posts: &Vec<Message>) -> Vec<ReplyTree> {
    if posts.len() == 0 {
        return Vec::new();
    }

    let mut root = ReplyTree::new(&posts[0]);

    for post in posts.iter().skip(1) {
        if let Some(data) = get_replydata(post) {
            if root.insert_post(post, &data).is_none() {
                println!(
                    "WARN: could not find place for message {}, reply to {}",
                    render::i(&post.id),
                    data.direct
                );
            }
        }
    }

    //println!("{:#?}", root);
    vec![root]
}

// --------------------------
// *   REQUEST GENERATION   *
// --------------------------

//"prepost" means the main query before finding the main data before gathering the posts. The post offset
//often depends on the prepost
pub fn get_prepost_request(
    fpid: Option<i64>,
    post_id: Option<i64>,
    ftid: Option<i64>,
    thread_hash: Option<String>,
) -> FullRequest {
    let mut request = FullRequest::new();

    let mut post_limited = false;
    let mut post_query = String::from("!basiccomments()");

    //If you call it with both, it will limit to both (chances are that's not what you want)
    if let Some(fpid) = fpid {
        add_value!(request, "fpidkey", vec!["fpid"]);
        add_value!(request, "fpid", vec![fpid]);
        //Remember: valuein way faster! eventually add "valueis"
        post_query.push_str(" and !valuein(@fpidkey, @fpid)");
        post_limited = true;
    }
    if let Some(post_id) = post_id {
        add_value!(request, "postId", post_id);
        post_query.push_str(" and id = @postId");
        post_limited = true;
    }

    add_value!(request, "allowed_types", THREADTYPES);
    let mut thread_query = String::from("!notdeleted() and literalType in @allowed_types");

    //Add the pre-lookup post get so we can limit the thread by it. This will prevent users
    //from sending random hashes but with valid post ids, since the thread won't be found
    if post_limited {
        let mut message_request = build_request!(
            RequestType::message,
            //Dont' need values for fpid, you already know it was there if it exists
            String::from("id,contentId"),
            post_query
        );
        message_request.limit = 1; //Just in case
        message_request.name = Some(String::from(PREMESSAGEKEY));
        request.requests.push(message_request);
        thread_query = format!("{} and id in @{}.contentId", thread_query, PREMESSAGEKEY);
    }

    //Take hashes over ftid if you gave both. Fail if neither are given
    if let Some(ftid) = ftid {
        add_value!(request, "ftidkey", vec!["ftid"]);
        add_value!(request, "ftid", vec![ftid]);
        thread_query = format!("{} and !valuein(@ftidkey, @ftid)", thread_query);
    } else if let Some(thread_hash) = thread_hash {
        add_value!(request, "hash", thread_hash);
        thread_query = format!("{} and hash = @hash", thread_query);
    } else if !post_limited {
        //Is this acceptable? I mean you called it wrong...
        panic!("You must pass at least one of either 'ftid' or 'thread_hash' or 'post_id' to get_prepost_request()!");
    }

    let mut thread_request = build_request!(
        RequestType::content,
        String::from("*"), //Here we ask for "everything" because we will be rendering all the thread data now
        thread_query
    );
    thread_request.expensive = true;
    thread_request.name = Some(String::from(THREADKEY));
    request.requests.push(thread_request);

    //And one last thing: you still need the category of course
    // let mut category_request = build_request!(
    //     RequestType::content,
    //     String::from(CATEGORYFIELDS),
    //     format!("!notdeleted() and id in @{}.parentId", THREADKEY) //format!("literalType = @category_literal and !notdeleted() and id in @{}.parentId", THREADKEY)
    // );
    // category_request.name = Some(String::from(CATEGORYKEY));
    // request.requests.push(category_request);

    //OK one last ACTUAL thing: need to get the premessage index if it was there
    if post_limited {
        let mut index_request = build_request!(
            RequestType::message,
            String::from("specialCount,id,contentId"),
            //This query DOES NOT fail if no premessage is found (like on user error). It needs to be LESS THAN
            //while ordered by id (default) to produce a proper index. The first message will be 0, and the second
            //will have one message with id lower than it.
            format!(
                "!basiccomments() and contentId in @{}.id and id < @{}.id",
                THREADKEY, PREMESSAGEKEY
            )
        );
        index_request.name = Some(String::from(PREMESSAGEINDEXKEY));
        request.requests.push(index_request);
    }

    request
}

fn get_generic_message_request(
    query: &str,
    extra_uids: Vec<i64>,
    limit: i32,
    skip: i32,
) -> FullRequest {
    let mut request = FullRequest::new();
    add_value!(request, "uids", extra_uids);

    let message_request = build_request!(
        RequestType::message,
        String::from("*"),
        format!("!basiccomments() and ({})", query),
        String::from("id"),
        limit,
        skip
    );
    request.requests.push(message_request);

    let mut related_request = build_request!(
        RequestType::message,
        String::from("*"),
        String::from("!basiccomments() and id in @message.values.re"),
        String::from("id")
    );
    related_request.name = Some(String::from("related"));
    request.requests.push(related_request);

    //users in messages OR in extra_uids
    let user_request = build_request!(
        RequestType::user,
        String::from("*"),
        String::from(
            "id in @message.createUserId or id in @message.editUserId or \
                      id in @related.createUserId or id in @related.editUserId or id in @uids"
        )
    );
    request.requests.push(user_request);

    request
}

//Apparently can't decide on transfered ownership or not
pub fn get_finishpost_request(
    thread_id: i64,
    extra_uids: Vec<i64>,
    limit: i32,
    skip: i32,
) -> FullRequest {
    let mut request =
        get_generic_message_request("contentId = @thread_id", extra_uids, limit, skip);
    add_value!(request, "thread_id", thread_id);
    request
}

/// Generate a request for ONLY messages and users for the given root post id. NO limits set on reply chain
/// length (other than those imposed by the API)
pub fn get_reply_request(root_post_id: i64) -> FullRequest {
    //NOTE: valuein WAY WAY faster than valuelike! always prefer it!
    let mut request = get_generic_message_request(
        "!valuein(@root_key,@root_post) or id = @root_post",
        Vec::new(),
        0,
        0,
    );
    add_value!(request, "root_key", vec!["re-top"]);
    add_value!(request, "root_post", vec![root_post_id]);
    request
}
