
use super::*;
use contentapi::*;
use contentapi::endpoints::*;
use serde_json::Value;
use crate::constants::*;
use contentapi::conversion::*;

// ------------------------------
//     CATEGORIES (FOR PAGES)
// ------------------------------

pub const CATEGORYFIELDS: &str = "id,literalType,contentType,values,name";

pub fn get_allcategory_query() -> String {
    format!("contentType = {{{{{}}}}} and !notdeleted() and literalType = {{{{{}}}}}", ContentType::SYSTEM, SBSPageType::CATEGORY)
}

pub async fn get_all_categories(context: &mut ApiContext, limit: Option<Vec<i64>>) -> Result<Vec<Content>, ApiError> //Box<dyn std::error::Error>>
{
    let mut request = FullRequest::new();

    request.requests.push(build_request!(
        RequestType::content,
        String::from(CATEGORYFIELDS),
        format!("{} {}", get_allcategory_query(), 
            if let Some(limit) = limit {
                add_value!(request, "limit", limit);
                " and id in @limit"
            } else { 
                "" 
            }
        )
    ));

    let result = context.post_request_profiled_opt(&request, "all_categories").await?;
    conversion::cast_result_required::<Content>(&result, &RequestType::content.to_string()).map_err(|e| e.into())
}

pub async fn get_content_vote(context: &ApiContext, content_id: i64) -> Result<Option<ContentEngagement>, ApiError>
{
    let mut request = FullRequest::new();
    add_value!(request, "contentId", content_id);
    add_value!(request, "upvote", UPVOTE);
    add_value!(request, "downvote", DOWNVOTE);
    add_value!(request, "type", VOTETYPE);
    let mut creq = build_request!(
        RequestType::content_engagement,
        String::from("*"),
        String::from("contentId = @contentId and type = @type and (engagement = @upvote or engagement = @downvote)")
    );
    creq.limit = 1; //Just in case
    request.requests.push(creq);

    let result = context.post_request(&request).await?;
    let mut engagement = conversion::cast_result_required::<ContentEngagement>(&result, &RequestType::content_engagement.to_string())?;
    Ok(engagement.pop())
}


// ---------------------------
//   SPECIAL SYSTEM CONTENT
// ---------------------------

pub async fn get_system_any(context: &mut ApiContext, ty: &str) -> Result<Option<Content>, Error> 
{
    let mut request = FullRequest::new();
    add_value!(request, "type", ContentType::SYSTEM);
    add_value!(request, "littype", ty);
    let alert_request = build_request!(
        RequestType::content,
        String::from("id,name,text,parentId,hash,contentType,literalType"),
        String::from("contentType = @type and literalType = @littype"),
        String::from("id") // Combined with 'pop', even if there are multiple alerts, we always get the last one
    );
    request.requests.push(alert_request);
    let result = context.post_request_profiled_opt(&request, "get-system").await?;
    let mut content = cast_result_required::<Content>(&result, "content")?;
    Ok(content.pop())
}

/// Returns the system alert; these should be in HTML format!
pub async fn get_system_alert(context: &mut ApiContext) -> Result<Option<Content>, Error> {
    get_system_any(context, SBSPageType::ALERT).await
}

/// Returns the frontpage; this shoudl be in HTML format!
pub async fn get_system_frontpage(context: &mut ApiContext) -> Result<Option<Content>, Error> {
    get_system_any(context, SBSPageType::FRONTPAGE).await
}


// ----------------------
//    GENERAL CONTENT
// ----------------------

#[derive(Default, Debug, Clone)]
pub struct FullPage {
    pub main: Content, 
    pub ptc: Option<Content>
}

pub async fn get_fullpage(context: &mut ApiContext, by_field: &str, value: Value) -> Result<FullPage, Error>
{
    let mut request = FullRequest::new();
    let notfound = Error::NotFound(format!("Could not find content with {} = {}", by_field, value));
    add_value!(request, "findby", value);
    add_value!(request, "ptcsystem", PTCSYSTEM);

    let mut main_request = build_request!(
        RequestType::content,
        String::from("*"),
        format!("{} = @findby", by_field)
    );
    main_request.limit = 1;
    main_request.name = Some(String::from("main"));
    request.requests.push(main_request);

    let mut ptc_request = build_request!(
        RequestType::content,
        String::from("*"),
        String::from("parentId = @main.id and literalType = @ptcsystem")
    );
    ptc_request.name = Some(String::from("ptc"));
    request.requests.push(ptc_request);

    let result = context.post_request_profiled_opt(&request, "page_everything").await?;
    let mut main = cast_result_required::<Content>(&result, "main")?;
    let mut ptc = cast_result_required::<Content>(&result, "ptc")?;

    Ok(FullPage {
        main: main.pop().ok_or(notfound)?,
        ptc : ptc.pop()
    })
}

pub async fn get_fullpage_by_hash(context: &mut ApiContext, hash: &str) -> Result<FullPage, Error>
{
    get_fullpage(context, "hash", hash.into()).await
}

pub async fn get_fullpage_by_id(context: &mut ApiContext, id: i64) -> Result<FullPage, Error>
{
    get_fullpage(context, "id", id.into()).await
}