use rocket_dyn_templates::Template;
use serde::Serialize;

use crate::context::*;
use crate::api_data::*;
use crate::api::*;
use crate::conversion;
use super::*;
use rocket::response::status::Custom as RocketCustom;


// Build a request for JUST forum categories
fn get_category_request() -> FullRequest
{
    //The request which we will spend the entire function building
    let mut request = FullRequest::new();
    add_value!(request, "category_literal", SBSContentType::forumcategory.to_string());

    let mut category_request = minimal_content!(String::from("literalType = @category_literal and !notdeleted()"));
    category_request.name = Some(String::from("category"));
    request.requests.push(category_request);

    request
}

// Build a request for thread data for the given forum category ids. This will produce
// individual queries for each category. This is one of the ONLY places where we need to
// perform such a decomposed and repetitious query: counting children of the thread parents
// ends up requiring permissions and it's not trivial to ask the API to do it. Comment counts
// are different because there can't be individual comments you can't see
fn get_thread_request(category_ids: &Vec<i64>, limit: i32) -> FullRequest
{
    let mut request = FullRequest::new();
    add_value!(request, "thread_literal", SBSContentType::forumthread.to_string());

    let mut comment_query = String::from("!basiccomments()");

    for category_id in category_ids 
    {
        //Standard threads get (for latest N threads)
        let base_query = format!("parentId = {{{{{category_id}}}}} and literalType = @thread_literal and !notedeleted()");
        let mut threads_request = minimal_content!(base_query.clone());
        let key = format!("threads_{}", category_id);

        threads_request.name = Some(key.clone());
        threads_request.order = Some(String::from("lastCommentId_desc"));
        threads_request.limit = limit.into(); 
        request.requests.push(threads_request);

        comment_query.push_str(" and id in @");
        comment_query.push_str(&key);
        comment_query.push_str(".lastCommentId");

        //Thread count get (if the previous is too expensive, consider just doing this)
        let mut count_request = build_request!(
            RequestType::content, 
            String::from("specialCount,parentId,literalType,id"), 
            base_query.clone()
        );
        count_request.name = Some(format!("threadcount_{}", category_id));
        request.requests.push(count_request);
    }

    let comment_request = minimal_message!(comment_query);
    request.requests.push(comment_request);

    println!("Threads request: {:?}", &request);

    request
}

#[derive(Serialize, Clone, Debug)]
struct ForumThread {
    thread: MinimalContent,
    posts_count: i32
}

//Structs JUST for building data for the forum templates (so no need to be public)
#[derive(Serialize, Clone, Debug)]
struct ForumCategory {
    category: MinimalContent,
    threads: Vec<ForumThread>,
    threads_count: i32
}

#[get("/forum")]
pub async fn forum_get(context: Context) -> Result<Template, RocketCustom<String>> 
{
    //First request: just get categories
    let request = get_category_request();
    let category_result = post_request(&context, &request).await.map_err(rocket_error!())?;
    let mut categories_raw = conversion::cast_result_required::<MinimalContent>(&category_result, "category").map_err(rocket_error!())?;

    //Next request: get the complicated dataset for each category
    let category_ids : Vec<i64> = categories_raw.iter().map(|catraw| catraw.id).collect();
    let thread_request = get_thread_request(&category_ids, context.config.default_recent_threads);
    let thread_result = post_request(&context, &thread_request).await.map_err(rocket_error!())?;

    //Sort the categories by their name AGAINST the default list in the config. So, it should sort the categories
    //by the order defined in the config, with stuff not present going at the end. Tiebreakers are resolved alphabetically
    categories_raw.sort_by_key(|catraw| {
        //Nicole made this a tuple so tiebreakers are sorted alphabetically, which is coool
        (context.config.forum_category_order.iter().position(|prefix| catraw.name.starts_with(prefix)).unwrap_or(usize::MAX), catraw.name.clone())
    });

    let mut categories = Vec::new();

    for catraw in categories_raw {
        let id = catraw.id;
        let threadcount_name = format!("threadcount_{}", id);
        let threads_name = format!("threads_{}", id);
        let special_counts = conversion::cast_result_required::<SpecialCount>(&thread_result, &threadcount_name).map_err(rocket_error!())?;
        let threads_raw = conversion::cast_result_required::<MinimalContent>(&thread_result, &threads_name).map_err(rocket_error!())?;

        let category = ForumCategory {
            category: catraw,
            threads: threads_raw.into_iter().map(|thread| ForumThread { 
                        thread,
                        posts_count: 0
                     }).collect(),
            threads_count: special_counts.get(0)
                .and_then(|sp| Some(sp.specialCount))
                .ok_or(ApiError::Precondition(format!("Didn't get specialCount for category {}", id)))
                .map_err(rocket_error!())?
        };
        categories.push(category);
    }

    println!("Template categories: {:?}", &categories);

    Ok(basic_template!("forum", context, {
        categories: categories
    }))
}
