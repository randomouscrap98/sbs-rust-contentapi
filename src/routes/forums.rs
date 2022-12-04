use std::collections::HashMap;

use rocket_dyn_templates::Template;
use serde::Serialize;
use anyhow::anyhow;

use crate::context::*;
use crate::api_data::*;
use crate::api::*;
use crate::conversion;
use super::*;

//To build the forum path at the top
#[derive(Serialize)]
struct ForumPath {
    link: String,
    title: String
}

impl ForumPath {
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


#[derive(Serialize, Clone, Debug)]
struct ForumThread {
    thread: Content,
    sticky: bool,
    posts: Vec<Message>
}

impl ForumThread {
    fn new(thread: Content, messages_raw: &Vec<Message>, stickies: &Vec<i64>) -> Self {
        let thread_id = thread.id;
        ForumThread { 
            thread,
            sticky: stickies.contains(&thread_id.unwrap_or(0)),
            posts: messages_raw.iter().filter(|m| m.contentId == thread_id).map(|m| m.clone()).collect()
        }
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

struct Keygen();

impl Keygen {
    fn threadcount(id: i64) -> String { format!("threadcount_{id}") }
    fn threads(id: i64) -> String { format!("threads_{id}") }
}


//Structs JUST for building data for the forum templates (so no need to be public)
#[derive(Serialize, Clone, Debug)]
struct ForumCategory {
    category: Content,
    threads: Vec<ForumThread>,
    threads_count: i32,
    users: HashMap<String, User>
}

impl ForumCategory {
    fn from_result(category: CleanedPreCategory, thread_result: &RequestResult, messages_raw: &Vec<Message>) -> Result<Self, anyhow::Error> {
        //let id = category.id.ok_or(anyhow!("Given forum category didn't have an id!"))?;
        let threadcount_name = Keygen::threadcount(category.id);
        let threads_name = Keygen::threads(category.id);

        let special_counts = conversion::cast_result_required::<SpecialCount>(&thread_result, &threadcount_name)?;
        let threads_raw = conversion::cast_result_required::<Content>(&thread_result, &threads_name)?;
        let users_raw = conversion::cast_result_required::<User>(&thread_result, "user")?;

        Ok(ForumCategory {
            category: category.category, //partial move
            threads: threads_raw.into_iter().map(|thread| ForumThread::new(thread, messages_raw, &category.stickies)).collect(),
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
        //Need values to know the stickies
        String::from("id,hash,name,description,literalType,values"),
        real_query);
    category_request.name = Some(String::from("category"));
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

    //let category_ids : Vec<i64> = categories.iter().map(|c| c.id).collect();
    let mut comment_query = String::from("!basiccomments() and (");
    let mut user_query = String::from("!notdeleted() and (id in @message.createUserId or ");
    let id_count = categories.len();
    //Not sure if we need values, but I NEED permissions to know if the thread is locked
    let fields = String::from("id,name,lastCommentId,literalType,hash,parentId,commentCount,createDate,values,permissions");

    for (index, ref category) in categories.iter().enumerate()
    {
        //Standard threads get (for latest N threads)
        let category_id = category.id;
        let base_query = format!("parentId = {{{{{category_id}}}}} and literalType = @thread_literal and !notdeleted()");
        let mut threads_request = build_request!(
            RequestType::content,
            fields.clone(),
            base_query.clone(),
            String::from("lastCommentId_desc"),
            limit,
            skip
        );
        let key = Keygen::threads(category_id);

        threads_request.name = Some(key.clone());
        request.requests.push(threads_request);

        comment_query = format!("{} id in @{}.lastCommentId", comment_query, &key);
        user_query = format!("{} id in @{}.createUserId", user_query, &key);

        //Only output 'or' if we're not at the end
        if index < id_count - 1 { 
            comment_query.push_str(" or "); 
            user_query.push_str(" or "); 
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

    comment_query.push_str(")");
    user_query.push_str(")");

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
    let categories_cleaned = CleanedPreCategory::from_many(conversion::cast_result_required::<Content>(&category_result, "category")?)?;
    let categories = build_categories(&context, categories_cleaned, 
        context.config.default_display_threads, 
        page * context.config.default_display_threads
    ).await?;

    let category = categories.get(0).ok_or(RouteError(rocket::http::Status::NotFound, String::from("Couldn't find that category")))?;

    println!("Please: {:?}", category);

    Ok(basic_template!("forumcategory", context, {
        //categories: categories
        category: category,
        page: page,
        forumpath: vec![ForumPath::root(), ForumPath::from_category(&category.category)]
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
    let mut categories_cleaned = CleanedPreCategory::from_many(conversion::cast_result_required::<Content>(&category_result, "category")?)?;

    //Sort the categories by their name AGAINST the default list in the config. So, it should sort the categories
    //by the order defined in the config, with stuff not present going at the end. Tiebreakers are resolved alphabetically
    categories_cleaned.sort_by_key(|category| {
        //Nicole made this a tuple so tiebreakers are sorted alphabetically, which is coool
        (context.config.forum_category_order.iter().position(
            |prefix| category.name.starts_with(prefix)).unwrap_or(usize::MAX), category.name.clone())
    });

    let categories = build_categories(&context, categories_cleaned, context.config.default_category_threads, 0).await?;

    println!("Template categories: {:?}", &categories);

    Ok(basic_template!("forum", context, {
        categories: categories,
        forumpath: vec![ForumPath::root()]
    }))
}

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