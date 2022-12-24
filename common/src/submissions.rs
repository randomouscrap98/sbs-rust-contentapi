use super::*;
use contentapi::*;
use crate::constants::*;


#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Search {
    pub search: Option<String>,
    pub order: String, 
    pub subtype: Option<String>, 
    pub system: String,
    pub category: Option<i64>,
    pub user_id: Option<i64>,
    pub removed: bool,
    pub page: i32
}

impl Default for Search {
    fn default() -> Self {
        Self {
            search: None,
            order: String::from(POPSCORE1SORT), 
            subtype: Some(String::from(SBSPageType::PROGRAM)), 
            system: String::from(ANYSYSTEM),
            user_id: None,
            category: None,
            removed: false, //By default, DON'T show removed!
            page: 0
        }
    }
}

// Generate the complicated FullRequest for the given search. Could be a "From" if 
// the search included a per-page I guess...
pub fn get_search_request(search: &Search, per_page: i32) -> FullRequest
{
    //Build up the request based on the search, then render
    let mut request = FullRequest::new();
    add_value!(request, "type", ContentType::PAGE);
    add_value!(request, "systemtype", ContentType::SYSTEM);
    add_value!(request, "forcontent", SBSValue::FORCONTENT);

    let mut query = String::from("contentType = @type and !notdeleted()"); 

    if let Some(stext) = &search.search {
        add_value!(request, "text", format!("%{}%", stext));
        query.push_str(" and (name like @text or !keywordlike(@text))");
    }

    if let Some(category) = search.category {
        if category != 0 {
            add_value!(request, "categoryTag", vec![format!("tag:{}", category)]);
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

    let main_request = build_request!(
        RequestType::content, 
        String::from("id,hash,contentType,literalType,values,name,description,createUserId,createDate,lastRevisionId,popScore1"), 
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

    add_value!(request, "categorytype", SBSPageType::CATEGORY);
    let mut category_request = build_request!(
        RequestType::content,
        String::from("id,literalType,contentType,values,name"),
        String::from("contentType = @systemtype and !notdeleted() and literalType = @categorytype") 
    );
    category_request.name = Some(String::from("categories"));
    request.requests.push(category_request);

    request
}
