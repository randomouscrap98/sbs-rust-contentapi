use std::collections::HashMap;

use super::*;
use contentapi::conversion::*;
use contentapi::*;


//Not sure if we need values, but I NEED permissions to know if the thread is locked
pub static THREADFIELDS : &str = "id,name,lastCommentId,literalType,contentType,hash,parentId,commentCount,createDate,createUserId,values,permissions";
//Need values to know the stickies
pub static CATEGORYFIELDS: &str = "id,hash,name,description,literalType,contentType,values";
pub static THREADKEY: &str = "thread";
pub static CATEGORYKEY: &str = "category";
pub static PREMESSAGEKEY: &str = "premessage";
pub static PREMESSAGEINDEXKEY: &str = "premessage_index";

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
        add_value!(request, "fcid_key", "fcid");
        add_value!(request, "fcid", fcid);
        real_query.push_str(" and !valuelike(@fcid_key, @fcid)");
    }
    else {
        //This is the "general" case, where yes, we actually do want to limit to categories. Otherwise,
        //if you pass a hash... it'll just work, regardless if it's a category or not.
        add_value!(request, "category_literals", vec![
            SBSContentType::forumcategory.to_string(),
            SBSContentType::submissions.to_string()
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
    //add_value!(request, "thread_literal", SBSContentType::forumthread.to_string());

    let mut keys = Vec::new();

    for ref category in categories.iter()
    {
        let category_id = category.id;
        let sticky_key = Keygen::stickies(category_id);
        request.values.insert(sticky_key.clone(), category.stickies.clone().into());

        //Standard threads get (for latest N threads)
        let base_query = format!("parentId = {{{{{category_id}}}}} and contentType = @page_type and !notdeleted()");

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
        add_value!(request, "fpid", fpid);
        add_value!(request, "fpidkey", "fpid");
        post_query.push_str(" and !valuelike(@fpidkey, @fpid)");
        post_limited = true;
    }
    if let Some(post_id) = post_id{
        add_value!(request, "postId", post_id);
        post_query.push_str(" and id = @postId");
        post_limited = true;
    }

    let mut thread_query = String::from("!notdeleted()");

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
        add_value!(request, "ftid", ftid);
        add_value!(request, "ftidkey", "ftid");
        thread_query = format!("{} and !valuelike(@ftidkey, @ftid)", thread_query);
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

//Apparently can't decide on transfered ownership or not
pub fn get_finishpost_request(thread_id: i64, extra_uids: Vec<i64>, limit: i32, skip: i32) -> FullRequest 
{
    let mut request = FullRequest::new();
    add_value!(request, "thread_id", thread_id);
    add_value!(request, "uids", extra_uids);

    let message_request = build_request!(
        RequestType::message,
        String::from("*"),
        String::from("!basiccomments() and contentId = @thread_id"),
        String::from("id"),
        limit,
        skip
    );
    request.requests.push(message_request);

    //users in messages OR in extra_uids
    let user_request = build_request!(
        RequestType::user,
        String::from("*"),
        String::from("id in @message.createUserId or id in @message.editUserId or id in @uids")
    );
    request.requests.push(user_request);

    request
}

/// Generate a request for ONLY messages and users for the given root post id. NO limits set on reply chain
/// length (other than those imposed by the API)
pub fn get_reply_request(root_post_id: i64) -> FullRequest 
{
    let mut request = FullRequest::new();
    //add_value!(request, "root_post", root_post_id);
    add_value!(request, "root_key", vec![format!("re:{}", root_post_id)]);

    let message_request = build_request!(
        RequestType::message,
        String::from("*"),
        String::from("!basiccomments() and !valuekeyin(@root_key)"),
        String::from("id")
    );
    request.requests.push(message_request);

    //users in messages OR in extra_uids
    let user_request = build_request!(
        RequestType::user,
        String::from("*"),
        String::from("id in @message.createUserId or id in @message.editUserId")
    );
    request.requests.push(user_request);

    request
}


// ----------------------------
// *     TEMPLATING PLUS      *
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
