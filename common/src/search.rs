use contentapi::*;
use crate::constants::*;
use crate::forms::*;
use crate::forum::can_delete_thread;
use crate::forum::can_edit_thread;
use crate::prefab::*;

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
            add_value!(request, "systemkey", SBSValue::SYSTEMS);
            add_value!(request, "subtype", subtype.clone());
            add_value!(request, "ptcsystem", format!("%{}%", PTCSYSTEM));
            query.push_str(" and literalType = @subtype");
            //Ignore certain search criteria
            if subtype == SBSPageType::PROGRAM {
                //MUST have a key unless the user specifies otherwise
                if !search.removed {
                    add_value!(request, "dlkeylist", vec![SBSValue::DOWNLOADKEY]);
                    query.push_str(" and (!valuekeyin(@dlkeylist) or !valuelike(@systemkey, @ptcsystem))");
                }

                if search.system != ANYSYSTEM {
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

//Both of these are the same as threads for now
pub fn can_edit_page(user: &User, page: &Content) -> bool { can_edit_thread(user, page) }
pub fn can_delete_page(user: &User, page: &Content) -> bool { can_delete_thread(user, page) }
