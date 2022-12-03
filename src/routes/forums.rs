use std::fmt::Display;

use rocket_dyn_templates::Template;
use serde::Serialize;

use crate::context::*;
use crate::api_data::*;
use crate::api::*;
use crate::conversion;
use super::*;
use rocket::response::status::Custom as RocketCustom;


fn get_category_request() -> FullRequest
{
    //The request which we will spend the entire function building
    let mut request = FullRequest::new();
    add_value!(request, "category_literal", SBSContentType::forumcategory.to_string());

    let mut category_request = minimal_content!(String::from("literalType = @category_literal"));
    category_request.name = Some(String::from("category"));
    request.requests.push(category_request);

    request
}

fn get_thread_request(category_ids: &Vec<i64>, limit: i32) -> FullRequest
{
    let mut request = FullRequest::new();
    add_value!(request, "thread_literal", SBSContentType::forumthread.to_string());

    for category_id in category_ids {
        let base_query = format!("parentId = {{{{{category_id}}}}} and literalType = @thread_literal");
        let mut threads_request = minimal_content!(base_query.clone());
        threads_request.name = Some(format!("threads_{}", category_id));
        threads_request.order = Some(String::from("lastCommentId"));
        threads_request.limit = limit.into(); //config.default_recent_threads.into();
        request.requests.push(threads_request);
        let mut count_request = build_request!(
            RequestType::content, 
            String::from("specialCount,parentId,literalType,id"), 
            base_query.clone()
        );
        count_request.name = Some(format!("threadcount_{}", category_id));
        request.requests.push(count_request);
    }

    request
}

//Perform the raw query for minimal data 
//async fn forumcategory_request(context: &Context) -> Result<RequestResult, ApiError>
//{
//    post_request(context, &request).await
//}

//async fn finalize_category_request(categories: &Vec<MinimalContent>, context: &Context) -> Result<Vec<ForumCategory>, impl Display>
//{
//    let mut request = FullRequest::new();
//    add_value!(request, "thread_literal", SBSContentType::forumthread.to_string());
//
//    for category in categories {
//        let mut threads_request = minimal_content!(String::from("parentId in @category.id and literalType = @thread_literal"));
//        threads_request.name = Some(format!("threads_{}", category_id));
//        threads_request.order = Some(String::from("lastCommentId"));
//        threads_request.limit = context.config.default_recent_threads.into();
//        request.requests.push(threads_request);
//    }
//
//    post_request(context, &request).await
//}

//async fn forumroot_request (context: &Context) -> Result<RequestResult, ApiError>
//{
//    //The request which we will spend the entire function building
//    let mut request = FullRequest::new();
//    add_value!(request, "category_literal", SBSContentType::forumcategory.to_string());
//    add_value!(request, "thread_literal", SBSContentType::forumthread.to_string());
//
//    let mut category_request = minimal_content!(String::from("literalType = @category_literal"));
//    category_request.name = Some(String::from("category"));
//    request.requests.push(category_request);
//
//    //Some awful hack to make a request per content type?
//    let mut threads_request = minimal_content!(String::from("parentId in @category.id and literalType = @thread_literal"));
//    threads_request.name = Some(String::from("threads"));
//    threads_request.order = Some(String::from("lastCommentId"));
//    threads_request.limit = context.config.default_recent_threads.into();
//    request.requests.push(threads_request);
//
//    ////But what if we were passed preview?
//    //if let Some(preview) = search.preview {
//    //    let hashes: Vec<String> = preview.split(",").map(|h| String::from(h.trim())).collect();
//    //    add_value!(request, "preview_hashes", hashes);
//    //    let mut preview_request = minimal_content!(format!("{} and hash in @preview_hashes", base_query));
//    //    preview_request.name = Some(String::from("preview"));
//    //    request.requests.push(preview_request);
//    //}
//
//    //println!("Sending: {:?}", &request);
//
//    post_request(context, &request).await
//}

#[derive(Serialize)]
struct ForumThread {
    thread: MinimalContent,

}

//Structs JUST for building data for the forum templates (so no need to be public)
#[derive(Serialize)]
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
    let mut categories_raw = conversion::cast_result::<MinimalContent>(&category_result, "category").map_err(rocket_error!())?;

    //Next request: get the complicated dataset for each category
    let category_ids : Vec<i64> = categories_raw.iter().map(|catraw| catraw.id).collect();
    let thread_request = get_thread_request(&category_ids, context.config.default_recent_threads);
    let thread_result = post_request(&context, &thread_request).await.map_err(rocket_error!())?;

    //let category_result = forumcategory_request(&context).await.map_err(rocket_error!())?;
    //let mut categories_raw = conversion::cast_result::<MinimalContent>(&category_result, "category").map_err(rocket_error!())?;
    //let mut categories = Vec::new();

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
        let threads_name = format!("threads{}", id);
        let special_counts = conversion::cast_result::<SpecialCount>(&thread_result, &threadcount_name).map_err(rocket_error!())?;
        let threads = conversion::cast_result::<MinimalContent>(&thread_result, &threads_name).map_err(rocket_error!())?;

        categories.push(ForumCategory {
            category: catraw,
            threads: threads.into_iter().map(|thread| ForumThread { thread }).collect(),
            threads_count: special_counts.get(0)
                .and_then(|sp| Some(sp.specialCount))
                .ok_or(ApiError::Precondition(format!("Didn't get specialCount for category {}", id)))
                .map_err(rocket_error!())?
        });
    }

    //let categories : Vec<ForumCategory> = categories_raw.into_iter().map(|catraw| {
    //}).collect();
    //ForumCategory {
    //    category: catraw,
    //    threads: Vec::new()
    //}).collect(); //.collect();

    Ok(basic_template!("forum", context, {
        categories: categories
    }))
}
