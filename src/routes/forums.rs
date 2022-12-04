use std::collections::HashMap;

use rocket_dyn_templates::Template;
use serde::Serialize;
use anyhow::anyhow;

use crate::context::*;
use crate::api_data::*;
use crate::api::*;
use crate::conversion;
use super::*;

//Not sure if we need values, but I NEED permissions to know if the thread is locked
static THREADFIELDS : &str = "id,name,lastCommentId,literalType,hash,parentId,commentCount,createDate,values,permissions";
//Need values to know the stickies
static CATEGORYFIELDS: &str = "id,hash,name,description,literalType,values";
static THREADKEY: &str = "thread";
static CATEGORYKEY: &str = "category";
static PREMESSAGEKEY: &str = "premessage";

struct Keygen();

impl Keygen {
    fn threadcount(id: i64) -> String { format!("threadcount_{id}") }
    fn threads(id: i64) -> String { format!("threads_{id}") }
    fn stickythreads(id: i64) -> String { format!("stickythreads_{id}") }
    fn stickies(id: i64) -> String { format!("stickies_{id}")}
}

//To build the forum path at the top
#[derive(Serialize)]
struct ForumPathItem {
    link: String,
    title: String
}

impl ForumPathItem {
    fn from_category(category: &Content) -> Self {
        Self {
            link: format!("/forum/category/{}", if let Some(ref hash) = category.hash { hash } else { "" }),
            title: if let Some(ref name) = category.name { name.clone() } else { String::from("NOTFOUND") }
        }
    }
    fn root() -> Self {
        Self {
            link: String::from("/forum"),
            title: String::from("Root")
        }
    }
}

#[derive(Serialize)]
struct ForumPagelistItem {
    text: String,
    current: bool,
    page: i32
}


#[derive(Serialize, Clone, Debug)]
struct ForumThread {
    thread: Content,
    sticky: bool,
    locked: bool,
    neutral: bool, //Used by the frontend
    posts: Vec<Message>
}

impl ForumThread {
    fn from_content(thread: Content, messages_raw: &Vec<Message>, stickies: &Vec<i64>) -> Result<Self, anyhow::Error> {
        let thread_id = thread.id;
        let permissions = match thread.permissions {
            Some(ref p) => Ok(p),
            None => Err(anyhow!("Thread didn't have permissions in resultset!"))
        }?;
        //"get" luckily already gets the thing as a reference
        let global_perms = permissions.get("0").ok_or(anyhow!("Thread didn't have global permissions!"))?;
        let locked = !global_perms.contains('C'); //Right... the order matters. need to finish using it before you give up thread
        let sticky = stickies.contains(&thread_id.unwrap_or(0));
        Ok(ForumThread { 
            locked, sticky, thread,
            neutral: !locked && !sticky,
            posts: messages_raw.iter().filter(|m| m.contentId == thread_id).map(|m| m.clone()).collect()
        })
    }
}

//Content is very lax with the fields, so we need something that will solidify SOME of them
//for use in other computations
struct CleanedPreCategory {
    category: Content,
    stickies: Vec<i64>,
    id: i64,
    name: String
}

impl CleanedPreCategory {
    fn from_content(category: Content) -> Result<CleanedPreCategory, anyhow::Error>{
        let name = match category.name {
            Some(ref n) => Ok(n.clone()),
            None => Err(anyhow!("Category search didn't have name!"))
        }?;
        let id = category.id.ok_or(anyhow!("Categories didn't have ids!"))?;
        //Need to get the list of stickies
        let cvalues = match category.values {
            Some(ref values) => Ok(values),
            None => Err(anyhow!("Given category didn't have values!"))
        }?;
        let sticky_value = cvalues.get("stickies").ok_or(anyhow!("Category didn't have stickes value!!"))?;
        let sticky_array = sticky_value.as_array().ok_or(anyhow!("Sticky wasn't array!"))?;
        let stickies = sticky_array.iter().map(|s| -> Result<i64, anyhow::Error> { s.as_i64().ok_or(anyhow!("Couldn't convert sticky value {}", s))}).collect::<Result<Vec<i64>, _>>()?;
        //let stickies = category.get_stickies()?;
        Ok(CleanedPreCategory { category: category, stickies, id, name })
    }

    fn from_many(categories: Vec<Content>) -> Result<Vec<CleanedPreCategory>, anyhow::Error> {
        categories.into_iter().map(|c| Self::from_content(c)).collect()
    }
}

//This struct is all the data to render a page in a single thread. Note that
//because of how complicated forum thread lookup is, this struct will be partially
//filled before completion
struct ThreadViewData {
    category: CleanedPreCategory,
    thread: ForumThread,
    users: HashMap<String, User>
}

//impl ThreadViewData {
//    fn prep_from_result(context: &Context, result: &RequestResult) -> Result<Self, anyhow::Error> {
//
//    }
//}


//Structs JUST for building data for the forum templates (so no need to be public)
#[derive(Serialize, Clone, Debug)]
struct ForumCategory {
    category: Content,
    threads: Vec<ForumThread>,
    stickies: Vec<ForumThread>,
    threads_count: i32,
    users: HashMap<String, User>
}

impl ForumCategory {
    fn from_result(category: CleanedPreCategory, thread_result: &RequestResult, messages_raw: &Vec<Message>) -> Result<Self, anyhow::Error> {
        //let id = category.id.ok_or(anyhow!("Given forum category didn't have an id!"))?;
        let threadcount_name = Keygen::threadcount(category.id);
        let threads_name = Keygen::threads(category.id);
        let stickies_name = Keygen::stickythreads(category.id);

        let special_counts = conversion::cast_result_required::<SpecialCount>(&thread_result, &threadcount_name)?;
        let threads_raw = conversion::cast_result_required::<Content>(&thread_result, &threads_name)?;
        let stickies_raw = conversion::cast_result_safe::<Content>(&thread_result, &stickies_name)?;
        let users_raw = conversion::cast_result_required::<User>(&thread_result, "user")?;

        Ok(ForumCategory {
            category: category.category, //partial move
            threads: threads_raw.into_iter().map(|thread| ForumThread::from_content(thread, messages_raw, &category.stickies)).collect::<Result<Vec<_>,_>>()?,
            stickies: stickies_raw.into_iter().map(|thread| ForumThread::from_content(thread, messages_raw, &category.stickies)).collect::<Result<Vec<_>,_>>()?,
            users: users_raw.into_iter().map(|u| (format!("{}", u.id), u)).collect(),
            threads_count: special_counts.get(0)
                .ok_or(ApiError::Usage(format!("Didn't get specialCount for category {}", category.id)))?.specialCount
        })
    }

}


// --------------------------
// *   REQUEST GENERATION   *
// --------------------------

// Build a request for JUST forum categories
fn get_category_request(hash: Option<String>, fcid: Option<i64>) -> FullRequest
{
    //The request which we will spend the entire function building
    let mut request = FullRequest::new();
    add_value!(request, "category_literal", SBSContentType::forumcategory.to_string());

    let mut real_query = String::from("literalType = @category_literal and !notdeleted()");

    if let Some(hash) = hash {
        add_value!(request, "hash", hash);
        real_query.push_str(" and hash = @hash");
    }
    else if let Some(fcid) = fcid {
        add_value!(request, "fcid_key", "fcid");
        add_value!(request, "fcid", fcid);
        real_query.push_str(" and !valuelike(@fcid_key, @fcid)");
    }

    let mut category_request = build_request!(RequestType::content, 
        String::from(CATEGORYKEY),
        real_query);
    category_request.name = Some(String::from(CATEGORYKEY));
    request.requests.push(category_request);

    request
}

// Build a request for thread data for the given forum category ids. This will produce
// individual queries for each category. This is one of the ONLY places where we need to
// perform such a decomposed and repetitious query: counting children of the thread parents
// ends up requiring permissions and it's not trivial to ask the API to do it. Comment counts
// are different because there can't be individual comments you can't see
fn get_thread_request(categories: &Vec<CleanedPreCategory>, limit: i32, skip: i32) -> FullRequest
{
    let mut request = FullRequest::new();
    add_value!(request, "thread_literal", SBSContentType::forumthread.to_string());

    let mut keys = Vec::new();

    for ref category in categories.iter()
    {
        let category_id = category.id;
        let sticky_key = Keygen::stickies(category_id);
        request.values.insert(sticky_key.clone(), category.stickies.clone().into());

        //Standard threads get (for latest N threads)
        let base_query = format!("parentId = {{{{{category_id}}}}} and literalType = @thread_literal and !notdeleted()");

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
        if skip == 0 {
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
            String::from("specialCount,parentId,literalType,id"), 
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

//fn add_prepost_content_request(mut request: &FullRequest, thread_query_extra: String) {
//
//}

//"prepost" means the main query before finding the main data before gathering the posts. The post offset
//often depends on the prepost
fn get_prepost_request(fpid: Option<i64>, post_id: Option<i64>, ftid: Option<i64>, thread_hash: Option<String>) -> FullRequest 
{
    let mut request = FullRequest::new();
    add_value!(request, "thread_literal", SBSContentType::forumthread.to_string());
    add_value!(request, "category_literal", SBSContentType::forumcategory.to_string());

    let mut post_limited = false;
    let mut post_query = String::from("!basiccomments()");
    let mut thread_query = format!("literalType = @thread_literal and !notdeleted()");

    //If you call it with both, it will limit to both (chances are that's not what you want)
    if let Some(fpid) = fpid {
        add_value!(request, "fpid", fpid);
        add_value!(request, "fpidkey", "fpid");
        post_query = format!("{} and !valuelike(@fpidkey, @fpid)", post_query);
        post_limited = true;
    }
    if let Some(post_id) = post_id{
        add_value!(request, "postId", post_id);
        post_query = format!("{} and id = @postId", post_query);
        post_limited = true;
    }

    //Add the pre-lookup post get so we can limit the thread by it. This will prevent users
    //from sending random hashes but with valid post ids, since the thread won't be found
    if post_limited {
        let mut message_request = build_request!(
            RequestType::message,
            //Dont' need values for fpid, you already know it was there if it exists
            String::from("id,contentId"),
            post_query
        );
        message_request.name = Some(String::from(PREMESSAGEKEY));
        request.requests.push(message_request);
        thread_query = format!("{} and id in @{}.contentId", thread_query, PREMESSAGEKEY);
    }

    //Limit thread lookup based on given params. You probably don't want both of these limits course
    if let Some(ftid) = ftid {
        add_value!(request, "ftid", ftid);
        add_value!(request, "ftidkey", "ftid");
        thread_query = format!("{} and !valuelike(@ftidkey, @ftid)", thread_query);
    }
    if let Some(thread_hash) = thread_hash {
        add_value!(request, "hash", thread_hash);
        thread_query = format!("{} and hash = @hash", thread_query);
    }

    let mut thread_request = build_request!(
        RequestType::content,
        String::from(THREADFIELDS),
        thread_query
    );
    thread_request.name = Some(String::from(THREADKEY));
    request.requests.push(thread_request);

    //And one last thing: you still need the category of course
    let mut category_request = build_request!(
        RequestType::content, 
        String::from(CATEGORYFIELDS),
        format!("literalType = @category_literal and !notdeleted() and id in @{}.parentId", THREADKEY)
    );
    category_request.name = Some(String::from(CATEGORYKEY));
    request.requests.push(category_request);

    request
}

//This is special because it needs many pieces of information, but not necessarily a bunch of 
//threads. The other thread request does complicated searches to get many threads; this one assumes
//a single thread with many posts. There is a chance the combination of data you give won't produce a thread
//fn get_posts_request(ftid: Option<i64>, fpid: Option<i64>, threadHash: Option<i64>, postId: Option<i64>, limit: i32, skip: i32) -> Result<FullRequest, anyhow::Error> {
//    let mut request = FullRequest::new();
//    add_value!(request, "thread_literal", SBSContentType::forumthread.to_string());
//    add_value!(request, "category_literal", SBSContentType::forumcategory.to_string());
//
//    //Can't get category or messages until you find the thread
//    let mut thread_query = format!("literalType = @thread_literal and !notdeleted()");
//
//    //It is an error to call this function with both fpid and postId, or ftid and threadHash
//    if ftid.is_some() && threadHash.is_some() {
//        return Err(anyhow!("You can't call get_posts_request with both an ftid and threadHash, they are simultaneous systems"));
//    }
//
//    //This is unfortunately complicated. If fpid or postId are given, we must lookup a message first.
//    if let Some(fpid) = fpid {
//        add_value!(request, "fpid", fpid);
//        add_value!(request, "fpidkey", "fpid");
//        let mut premessage_request = build_request!(
//            RequestType::message,
//            String::from("id,contentId"),
//            format!("!valuelike(@fpidkey, @fpid) and !basiccomments()")
//        );
//        premessage_request.name = Some(String::from(PREMESSAGEKEY));
//        //The thread needs to be the parent of the given fpid
//        thread_query = format!("{} and id in @{}.contentId", thread_query, PREMESSAGEKEY);
//    }
//
//
//    //Regular thread request. Needs to specifically NOT be the stickies
//    let mut threads_request = build_request!(
//        RequestType::content,
//        String::from(THREADFIELDS),
//        format!("{} and id not in @{}", base_query, sticky_key),
//        String::from("lastCommentId_desc"),
//        limit,
//        skip
//    );
//
//        let key = Keygen::threads(category_id);
//        threads_request.name = Some(key.clone());
//        request.requests.push(threads_request);
//
//    Ok(request)
//}

// --------------------------
// *    FORUM FUNCTIONS     *
// --------------------------

async fn build_categories(context: &Context, categories_cleaned: Vec<CleanedPreCategory>, limit: i32, skip: i32) -> Result<Vec<ForumCategory>, anyhow::Error> {
    //Next request: get the complicated dataset for each category (this somehow includes comments???)
    let thread_request = get_thread_request(&categories_cleaned, limit, skip); //context.config.default_category_threads, 0);
    let thread_result = post_request(context, &thread_request).await?;

    let messages_raw = conversion::cast_result_required::<Message>(&thread_result, "message")?;

    let mut categories = Vec::new();

    for category in categories_cleaned {
        categories.push(ForumCategory::from_result(category, &thread_result, &messages_raw)?);
    }

    Ok(categories)
}


async fn render_threads(context: &Context, category_request: FullRequest, page: Option<i32>) -> Result<Template, RouteError>
{
    let page = page.unwrap_or(0);

    let category_result = post_request(context, &category_request).await?;
    let categories_cleaned = CleanedPreCategory::from_many(conversion::cast_result_required::<Content>(&category_result, CATEGORYKEY)?)?;
    let categories = build_categories(&context, categories_cleaned, 
        context.config.default_display_threads, 
        page * context.config.default_display_threads
    ).await?;

    let category = categories.get(0).ok_or(RouteError(rocket::http::Status::NotFound, String::from("Couldn't find that category")))?;
    let mut pagelist = Vec::new();

    for i in (0..category.threads_count).step_by(context.config.default_display_threads as usize) {
        let thispage = i / context.config.default_display_threads;
        pagelist.push(ForumPagelistItem {
            page: thispage,
            text: format!("{}", thispage + 1),
            current: thispage == page
        });
    }

    //println!("Please: {:?}", category);

    Ok(basic_template!("forumcategory", context, {
        //categories: categories
        category: category,
        page: page,
        pagelist: pagelist,
        forumpath: vec![ForumPathItem::root(), ForumPathItem::from_category(&category.category)]
    }))
}


async fn render_thread(context: &Context, pre_request: FullRequest, page: Option<i32>) -> Result<Template, RouteError> 
{
    let page = page.unwrap_or(0);

    let pre_result = post_request(context, &pre_request).await?;

    Ok(basic_template!("forumthread", context, {

    }))
}


// ----------------------
// *       ROUTES       *
// ----------------------

#[get("/forum")]
pub async fn forum_get(context: Context) -> Result<Template, RouteError> 
{
    //First request: just get categories
    let request = get_category_request(None, None);
    let category_result = post_request(&context, &request).await?;
    let mut categories_cleaned = CleanedPreCategory::from_many(conversion::cast_result_required::<Content>(&category_result, CATEGORYKEY)?)?;

    //Sort the categories by their name AGAINST the default list in the config. So, it should sort the categories
    //by the order defined in the config, with stuff not present going at the end. Tiebreakers are resolved alphabetically
    categories_cleaned.sort_by_key(|category| {
        //Nicole made this a tuple so tiebreakers are sorted alphabetically, which is coool
        (context.config.forum_category_order.iter().position(
            |prefix| category.name.starts_with(prefix)).unwrap_or(usize::MAX), category.name.clone())
    });

    let categories = build_categories(&context, categories_cleaned, context.config.default_category_threads, 0).await?;

    //println!("Template categories: {:?}", &categories);

    Ok(basic_template!("forum", context, {
        categories: categories,
        forumpath: vec![ForumPathItem::root()]
    }))
}

// Category view (list threads)
// ----------------------------

#[get("/forum/category/<hash>?<page>")]
pub async fn forum_categoryhash_get(context: Context, hash: String, page: Option<i32>) -> Result<Template, RouteError> 
{
    render_threads(&context, get_category_request(Some(hash), None), page).await
}

#[get("/forum?<fcid>&<page>", rank=2)]
pub async fn forum_categoryfcid_get(context: Context, fcid: i64, page: Option<i32>) -> Result<Template, RouteError> 
{
    render_threads(&context, get_category_request(None, Some(fcid)), page).await
}

// Thread view (list posts)
// ----------------------------
#[get("/forum/thread/<hash>?<page>")]
pub async fn forum_threadhash_get(context: Context, hash: String, page: Option<i32>) -> Result<Template, RouteError> 
{
    render_thread(&context, get_prepost_request(None, None, None, Some(hash)), page).await
}

#[get("/forum/thread/<hash>/<post_id>")]
pub async fn forum_threadhash_postid_get(context: Context, hash: String, post_id: i64) -> Result<Template, RouteError> 
{
    render_thread(&context, get_prepost_request(None, Some(post_id), None, Some(hash)), None).await
}

#[get("/forum?<ftid>&<page>", rank=3)]
pub async fn forum_thread_ftid_get(context: Context, ftid: i64, page: Option<i32>) -> Result<Template, RouteError> 
{
    render_thread(&context, get_prepost_request(None, None, Some(ftid), None), page).await
}

#[get("/forum?<fpid>", rank=4)]
pub async fn forum_thread_fpid_get(context: Context, fpid: i64) -> Result<Template, RouteError> 
{
    render_thread(&context, get_prepost_request(Some(fpid), None, None, None), None).await
}