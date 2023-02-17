
use contentapi::*;
use contentapi::endpoints::ApiContext;
use contentapi::endpoints::ApiError;
//use contentapi::endpoints::ApiContext;
//use contentapi::endpoints::ApiError;
use crate::constants::*;
use crate::forms::*;
use crate::forum::can_delete_thread;
use crate::forum::can_edit_thread;

pub const CATEGORYPREFIX: &str = "tag:";
//pub const CATEGORYSEARCHBASE: &str = "contentType = @systemtype and !notdeleted() and literalType = {{}}";
pub const CATEGORYFIELDS: &str = "id,literalType,contentType,values,name";

pub fn get_allcategory_query() -> String {
    format!("contentType = {{{{{}}}}} and !notdeleted() and literalType = {{{{{}}}}}", ContentType::SYSTEM, SBSPageType::CATEGORY)
}

/// Get the list of category ids this content is tagged under
pub fn get_tagged_categories(content: &Content) -> Vec<i64>
{
    let mut result : Vec<i64> = Vec::new();

    if let Some(ref values) = content.values {
        for (key, _value) in values {
            if key.starts_with(CATEGORYPREFIX) {
                if let Ok(category) = (&key[CATEGORYPREFIX.len()..]).parse::<i64>() {
                    result.push(category)
                }
            }
        }
    }

    result
}

/// Add a parsed list of categories from a user form (which should be just ids)
/// to the given content. It will add them as values
pub fn add_category_taglist(raw_parsed: Vec<String>, content: &mut Content)
{
    if let Some(ref mut values) = content.values {
        for category in raw_parsed {
            values.insert(format!("{}{}", CATEGORYPREFIX, category), true.into());
        }
    }
}

/// Generate the complicated FullRequest for the given search. Could be a "From" if 
/// the search included a per-page I guess...
pub fn get_search_request(search: &PageSearch, per_page: i32) -> FullRequest
{
    //Build up the request based on the search, then render
    let mut request = FullRequest::new();
    add_value!(request, "type", ContentType::PAGE);
    add_value!(request, "systemtype", ContentType::SYSTEM);
    add_value!(request, "forcontent", SBSValue::FORCONTENT);
    add_value!(request, "submissions_type", SBSPageType::SUBMISSIONS);

    let mut parent_request = build_request!(
        RequestType::content, 
        String::from("id,literalType,contentType"), 
        String::from("literalType = @submissions_type and contentType = @systemtype")
    ); 
    parent_request.name = Some("submissions".to_string());
    request.requests.push(parent_request);

    let mut query = String::from("contentType = @type and !notdeleted() and parentId in @submissions.id"); 

    if let Some(stext) = &search.search {
        add_value!(request, "text", format!("%{}%", stext));
        query.push_str(" and (name like @text or !keywordlike(@text))");
    }

    if let Some(category) = search.category {
        if category != 0 {
            add_value!(request, "categoryTag", vec![format!("{}{}", CATEGORYPREFIX, category)]);
            query.push_str(" and !valuekeyin(@categoryTag)");
        }
    }

    if let Some(user_id) = search.user_id {
        if user_id != 0 {
            add_value!(request, "userId", user_id);
            query.push_str(" and createUserId = @userId");
        }
    }

    // This special request generator can be used in a lot of contexts, so there's lots of optional
    // fields. The system doesn't HAVE to limit by subtype (program/resource/etc)
    if let Some(subtype) = &search.subtype 
    {
        if !subtype.is_empty() {
            add_value!(request, "subtype", subtype.clone());
            query.push_str(" and literalType = @subtype");
            //Ignore certain search criteria
            if subtype == SBSPageType::PROGRAM {
                //MUST have a key unless the user specifies otherwise
                if !search.removed {
                    add_value!(request, "dlkeylist", vec![SBSValue::DOWNLOADKEY]);
                    query.push_str(" and !valuekeyin(@dlkeylist)");
                }

                if search.system != ANYSYSTEM {
                    add_value!(request, "systemkey", SBSValue::SYSTEMS);
                    add_value!(request, "system", format!("%{}%", search.system)); //Systems is actually a json list but this should be fine
                    query.push_str(" and !valuelike(@systemkey, @system)");
                }
            }
        }
    }

    let main_request = build_request!(
        RequestType::content, 
        String::from("id,hash,parentId,contentType,literalType,values,name,description,createUserId,createDate,lastRevisionId,popScore1"), 
        query, 
        search.order.clone(), 
        per_page,
        search.page * per_page
    ); 
    request.requests.push(main_request);

    let user_request = build_request!(
        RequestType::user,
        String::from("*"),
        String::from("id in @content.createUserId")
    );
    request.requests.push(user_request);

    //add_value!(request, "categorytype", SBSPageType::CATEGORY);
    let mut category_request = build_request!(
        RequestType::content,
        String::from(CATEGORYFIELDS),
        get_allcategory_query()
        //format!("{} and literalType = @categorytype", CATEGORYSEARCHBASE) 
    );
    category_request.name = Some(String::from("categories"));
    request.requests.push(category_request);

    request
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

#[derive(Debug)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub forcontent: String
}

pub fn map_categories(categories: Vec<Content>) -> Vec<Category>
{
    categories.into_iter().map(|c| {
        Category {
            id: c.id.unwrap_or(0),
            name: c.name.unwrap_or_else(|| String::from("")), //Only evaluated on failure
            forcontent: c.values
                .and_then(|v| v.get(SBSValue::FORCONTENT).and_then(|v2| v2.as_str()).and_then(|v3| Some(String::from(v3))))
                .unwrap_or_else(|| String::from(""))
        }
    }).collect::<Vec<Category>>()
}

//Both of these are the same as threads for now
pub fn can_edit_page(user: &User, page: &Content) -> bool { can_edit_thread(user, page) }
pub fn can_delete_page(user: &User, page: &Content) -> bool { can_delete_thread(user, page) }