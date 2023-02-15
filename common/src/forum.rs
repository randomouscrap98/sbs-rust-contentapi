use std::collections::HashMap;

use super::*;
use crate::constants::*;

use contentapi::conversion::*;
use contentapi::*;
use contentapi::permissions::can_user_action;


//Not sure if we need values, but I NEED permissions to know if the thread is locked
pub static THREADFIELDS : &str = "id,name,lastCommentId,literalType,contentType,hash,parentId,commentCount,createDate,createUserId,values,permissions";
//Need values to know the stickies
pub static CATEGORYFIELDS: &str = "id,hash,name,description,literalType,contentType,values";

//Note: these are keys for the REQUESTS, not anything else!
pub static THREADKEY: &str = "thread";
pub static CATEGORYKEY: &str = "category";
pub static PREMESSAGEKEY: &str = "premessage";
pub static PREMESSAGEINDEXKEY: &str = "premessage_index";

pub const ALLOWEDTYPES: &[&str] = &[
    SBSPageType::FORUMTHREAD,
    SBSPageType::PROGRAM,
    SBSPageType::RESOURCE,
    SBSPageType::DIRECTMESSAGE
];

struct Keygen();

impl Keygen {
    fn threadcount(id: i64) -> String { format!("threadcount_{id}") }
    fn threads(id: i64) -> String { format!("threads_{id}") }
    fn stickythreads(id: i64) -> String { format!("stickythreads_{id}") }
    fn stickies(id: i64) -> String { format!("stickies_{id}")}
}

#[derive(Clone, Debug)]
pub struct ForumThread {
    pub thread: Content,
    pub sticky: bool,
    pub locked: bool,
    pub private: bool,
    pub neutral: bool, //Used by the frontend
    pub posts: Vec<Message>
}

impl ForumThread {
    pub fn from_content(thread: Content, messages_raw: &Vec<Message>, stickies: &Vec<i64>) -> Result<Self, Error> {
        let thread_id = thread.id;
        let permissions = match thread.permissions {
            Some(ref p) => Ok(p),
            None => Err(Error::Other(String::from("Thread didn't have permissions in resultset!")))
        }?;
        //"get" luckily already gets the thing as a reference
        //These are APPROXIMATIONS for display only! They should NOT be used to determine ACTUAL functionality!
        let global_perms = permissions.get("0").and_then(|s| Some(s.as_str())).unwrap_or_else(||"");
        //ok_or(Error::Other(String::from("Thread didn't have global permissions!")))?;
        let locked = !global_perms.contains('C'); //Right... the order matters. need to finish using it before you give up thread
        let private = !global_perms.contains('R');
        let sticky = stickies.contains(&thread_id.unwrap_or(0));
        Ok(ForumThread { 
            locked, sticky, thread, private,
            neutral: !locked && !sticky,
            posts: messages_raw.iter().filter(|m| m.contentId == thread_id).map(|m| m.clone()).collect()
        })
    }
}

//Structs JUST for building data for the forum templates (so no need to be public)
#[derive(Clone, Debug)]
pub struct ForumCategory {
    pub category: Content,
    pub threads: Vec<ForumThread>,
    pub stickies: Vec<ForumThread>,
    pub threads_count: i32,
    pub users: HashMap<i64, User>
}

impl ForumCategory {
    pub fn from_result(category: CleanedPreCategory, thread_result: &RequestResult, messages_raw: &Vec<Message>) -> Result<Self, Error> {
        //let id = category.id.ok_or(anyhow!("Given forum category didn't have an id!"))?;
        let threadcount_name = Keygen::threadcount(category.id);
        let threads_name = Keygen::threads(category.id);
        let stickies_name = Keygen::stickythreads(category.id);

        let special_counts = cast_result_required::<SpecialCount>(&thread_result, &threadcount_name)?;
        let threads_raw = cast_result_required::<Content>(&thread_result, &threads_name)?;
        let stickies_raw = cast_result_safe::<Content>(&thread_result, &stickies_name)?;
        let users_raw = cast_result_required::<User>(&thread_result, "user")?;

        Ok(ForumCategory {
            category: category.category, //partial move
            threads: threads_raw.into_iter().map(|thread| ForumThread::from_content(thread, messages_raw, &category.stickies)).collect::<Result<Vec<_>,_>>()?,
            stickies: stickies_raw.into_iter().map(|thread| ForumThread::from_content(thread, messages_raw, &category.stickies)).collect::<Result<Vec<_>,_>>()?,
            users: map_users(users_raw),
            threads_count: special_counts.get(0)
                .ok_or(Error::Data(format!("Didn't get specialCount for category {}", category.id), format!("{:?}", thread_result)))?.specialCount
        })
    }
}

//Content is very lax with the fields, so we need something that will solidify SOME of them
//for use in other computations
pub struct CleanedPreCategory {
    pub category: Content,
    pub stickies: Vec<i64>,
    pub id: i64,
    pub name: String
}

impl CleanedPreCategory {
    pub fn from_content(category: Content) -> Result<CleanedPreCategory, Error>{
        let name = match category.name {
            Some(ref n) => Ok(n.clone()),
            None => Err(Error::Other(String::from("Category search didn't have name!")))
        }?;
        let id = category.id.ok_or(Error::Other(String::from("Categories didn't have ids!")))?;
        //Need to get the list of stickies
        let cvalues = match category.values {
            Some(ref values) => Ok(values),
            None => Err(Error::Other(String::from("Given category didn't have values!")))
        }?;
        let stickies;
        //it is OK for something to not have stickied threads! 
        if let Some(sticky_value) = cvalues.get("stickies") { //}.ok_or(Error::Other(String::from("Category didn't have stickies value!!")))?;
            let sticky_array = sticky_value.as_array().ok_or(Error::Other(String::from("Sticky wasn't array!")))?;
            stickies = sticky_array.iter().map(|s| -> Result<i64, Error> { 
                s.as_i64().ok_or(Error::Other(format!("Couldn't convert sticky value {}", s)))
            }).collect::<Result<Vec<i64>, _>>()?;
        }
        else {
            stickies = Vec::new();
        }
        //let stickies = category.get_stickies()?;
        Ok(CleanedPreCategory { category: category, stickies, id, name })
    }

    pub fn from_many(categories: Vec<Content>) -> Result<Vec<CleanedPreCategory>, Error> {
        categories.into_iter().map(|c| Self::from_content(c)).collect()
    }
}


pub struct ReplyData {
    pub top: i64,
    pub direct: i64
}

impl ReplyData {
    pub fn write_to_values(&self, values: &mut HashMap<String, serde_json::Value>) {
        values.insert(String::from("re-top"), self.top.into());
        values.insert(String::from("re"), self.direct.into());
    }
}

/// Compute the flattened reply data for the given message
pub fn get_replydata(post: &Message) -> Option<ReplyData>
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

/// Given a post, regenerate the new reply data that would properly point to this post
pub fn get_new_replydata(post: &Message) -> ReplyData
{
    let id = post.id.unwrap_or_default();

    let mut reply_data = ReplyData {
        top: id,
        direct: id
    };

    //Oh but if we can parse existing reply data off the message, that one's top becomes our top too
    if let Some(existing) = get_replydata(post) {
        reply_data.top = existing.top;
    }

    reply_data
}

pub struct ReplyTree<'a> {
    pub id: i64,
    pub post: &'a Message,
    pub children: Vec<ReplyTree<'a>>
}

impl<'a> ReplyTree<'a> {
    pub fn new(message: &'a Message) -> Self {
        ReplyTree { 
            id: message.id.unwrap_or_else(||0), 
            post: message, 
            children: Vec::new() 
        }
    }
}

pub fn posts_to_replytree(posts: &Vec<Message>) -> Vec<ReplyTree> 
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
            println!("WARN: could not find place for message {}, reply to {}", render::i(&post.id), data.direct);
        }
        temp_tree.push(ReplyTree::new(post));
    }
    temp_tree
}


// --------------------------
// *   REQUEST GENERATION   *
// --------------------------

// Build a request for JUST forum categories
pub fn get_category_request(hash: Option<String>, fcid: Option<i64>) -> FullRequest
{
    //The request which we will spend the entire function building
    let mut request = FullRequest::new();

    let mut real_query = String::from("!notdeleted()");

    if let Some(hash) = hash {
        add_value!(request, "hash", hash);
        real_query.push_str(" and hash = @hash");
    }
    else if let Some(fcid) = fcid {
        add_value!(request, "fcid_key", vec!["fcid"]);
        add_value!(request, "fcid", vec![fcid]);
        real_query.push_str(" and !valuein(@fcid_key, @fcid)");
    }
    else {
        //This is the "general" case, where yes, we actually do want to limit to categories. Otherwise,
        //if you pass a hash... it'll just work, regardless if it's a category or not.
        add_value!(request, "category_literals", vec![
            SBSPageType::FORUMCATEGORY,
            SBSPageType::SUBMISSIONS
        ]);
        real_query.push_str(" and literalType in @category_literals");
    }

    let mut category_request = build_request!(
        RequestType::content, 
        String::from(CATEGORYFIELDS),
        real_query
    );
    category_request.name = Some(String::from(CATEGORYKEY));
    request.requests.push(category_request);

    request
}


pub fn get_thread_request(categories: &Vec<CleanedPreCategory>, limit: i32, skip: i32, get_stickies: bool) -> FullRequest
{
    let mut request = FullRequest::new();
    add_value!(request, "page_type", ContentType::PAGE);
    add_value!(request, "allowed_types", ALLOWEDTYPES);

    let mut keys = Vec::new();

    for ref category in categories.iter()
    {
        let category_id = category.id;
        let sticky_key = Keygen::stickies(category_id);
        request.values.insert(sticky_key.clone(), category.stickies.clone().into());

        //Standard threads get (for latest N threads)
        let base_query = format!("parentId = {{{{{category_id}}}}} and contentType = @page_type and literalType in @allowed_types and !notdeleted()");

        //Regular thread request. Needs to specifically NOT be the stickies
        let mut threads_request = build_request!(
            RequestType::content,
            String::from(THREADFIELDS),
            format!("{} and id not in @{}", base_query, sticky_key),
            String::from("lastCommentId_desc"),
            limit,
            skip
        );

        let key = Keygen::threads(category_id);
        threads_request.name = Some(key.clone());
        request.requests.push(threads_request);
        keys.push(key);

        // NO limits on sticky request. The "only if no skip" might not be great
        if skip == 0 && get_stickies {
            let mut sticky_request = build_request!(
                RequestType::content,
                String::from(THREADFIELDS),
                format!("{} and id in @{}", base_query, sticky_key),
                String::from("lastCommentId_desc")
            );

            let key = Keygen::stickythreads(category_id);
            sticky_request.name = Some(key.clone());
            request.requests.push(sticky_request);
            keys.push(key);
        }

        //Thread count get (if the previous is too expensive, consider just doing this)
        let mut count_request = build_request!(
            RequestType::content, 
            String::from("specialCount,parentId,literalType,contentType,id"), 
            base_query.clone()
        );
        count_request.name = Some(Keygen::threadcount(category_id));
        request.requests.push(count_request);
    }

    //How many string allocations is this? I mean it shouldn't matter but ugh
    let comment_query = format!("!basiccomments() and ({})", 
        keys.iter().map(|k| format!("id in @{}.lastCommentId", k)).collect::<Vec<String>>().join(" or "));
    let user_query = format!("!notdeleted() and (id in @message.createUserId or {})",
        keys.iter().map(|k| format!("id in @{}.createUserId", k)).collect::<Vec<String>>().join(" or "));

    let comment_request = build_request!(
        RequestType::message,
        String::from("id,createDate,contentId,createUserId"),
        comment_query);
    request.requests.push(comment_request);

    let user_request = build_request!(
        RequestType::user,
        String::from("*"),
        user_query);
    request.requests.push(user_request);

    //println!("Threads request: {:?}", &request);

    request
}

//"prepost" means the main query before finding the main data before gathering the posts. The post offset
//often depends on the prepost
pub fn get_prepost_request(fpid: Option<i64>, post_id: Option<i64>, ftid: Option<i64>, thread_hash: Option<String>) -> FullRequest 
{
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
    if let Some(post_id) = post_id{
        add_value!(request, "postId", post_id);
        post_query.push_str(" and id = @postId");
        post_limited = true;
    }

    add_value!(request, "allowed_types", ALLOWEDTYPES);
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
    }
    else if let Some(thread_hash) = thread_hash {
        add_value!(request, "hash", thread_hash);
        thread_query = format!("{} and hash = @hash", thread_query);
    }
    else if !post_limited {
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
    let mut category_request = build_request!(
        RequestType::content, 
        String::from(CATEGORYFIELDS),
        format!("!notdeleted() and id in @{}.parentId", THREADKEY)
        //format!("literalType = @category_literal and !notdeleted() and id in @{}.parentId", THREADKEY)
    );
    category_request.name = Some(String::from(CATEGORYKEY));
    request.requests.push(category_request);

    //OK one last ACTUAL thing: need to get the premessage index if it was there
    if post_limited {
        let mut index_request = build_request!(
            RequestType::message,
            String::from("specialCount,id,contentId"),
            //This query DOES NOT fail if no premessage is found (like on user error). It needs to be LESS THAN
            //while ordered by id (default) to produce a proper index. The first message will be 0, and the second
            //will have one message with id lower than it.
            format!("!basiccomments() and contentId in @{}.id and id < @{}.id", THREADKEY, PREMESSAGEKEY)
        );
        index_request.name = Some(String::from(PREMESSAGEINDEXKEY));
        request.requests.push(index_request);
    }

    request
}

fn get_generic_message_request(query: &str, extra_uids: Vec<i64>, limit: i32, skip: i32) -> FullRequest 
{
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
        String::from("id in @message.createUserId or id in @message.editUserId or \
                      id in @related.createUserId or id in @related.editUserId or id in @uids")
    );
    request.requests.push(user_request);

    request
}

//Apparently can't decide on transfered ownership or not
pub fn get_finishpost_request(thread_id: i64, extra_uids: Vec<i64>, limit: i32, skip: i32) -> FullRequest 
{
    let mut request = get_generic_message_request("contentId = @thread_id", extra_uids, limit, skip);
    add_value!(request, "thread_id", thread_id);
    request
}

/// Generate a request for ONLY messages and users for the given root post id. NO limits set on reply chain
/// length (other than those imposed by the API)
pub fn get_reply_request(root_post_id: i64) -> FullRequest 
{
    //NOTE: valuein WAY WAY faster than valuelike! always prefer it!
    let mut request = get_generic_message_request("!valuein(@root_key,@root_post) or id = @root_post", Vec::new(), 0, 0);
    add_value!(request, "root_key", vec!["re-top"]);
    add_value!(request, "root_post", vec![root_post_id]);
    request
}

//------------------
//   PERMISSIONS
//------------------

pub fn can_edit_thread(user: &User, thread: &Content) -> bool
{
    can_user_action(user, "U", thread)
}

pub fn can_delete_thread(user: &User, _thread: &Content) -> bool
{
    //NOTE: we WERE going to have groups and all that for "moderators" but no more: apparently contentapi
    //doesn't let you modify comments/posts unless you're super, which means moderators would HAVE to 
    //be super users anyway. As such, since we're only allowing "mods" to delete threads, this check
    //is sufficient
    user.admin
}

pub fn can_create_post(user: &User, thread: &Content) -> bool
{
    can_user_action(user, "C", thread)
}