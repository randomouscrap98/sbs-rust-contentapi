use super::*;
use contentapi::*;

pub static DOWNLOADKEYKEY: &str = "dlkey";
pub static VERSIONKEY: &str = "version";
pub static SIZEKEY : &str = "size";
pub static SYSTEMSKEY: &str = "systems";
pub static IMAGESKEY: &str = "images";
pub static FORCONTENTKEY: &str = "forcontent";
 
pub static CATEGORYTYPE: &str = "category";
pub static PROGRAMTYPE: &str = "program";
pub static RESOURCETYPE: &str = "resource";
 
pub static POPSCORE1SORT: &str = "popScore1_desc";
pub static ANYSYSTEM: &str = "any";

pub enum SubmissionSystem { }

impl SubmissionSystem {
    pub fn list() -> Vec<(&'static str, &'static str)> {
        //Idk, whatever
        vec![
            (ANYSYSTEM, "Any"), 
            ("3ds", "Nintendo 3DS"), 
            ("wiiu", "Nintendo WiiU"), 
            ("switch", "Nintendo Switch")
        ].into_iter().collect()
    }
}

pub enum SubmissionType { }

impl SubmissionType {
    pub fn list() -> HashMap<&'static str, &'static str> {
        //Idk, whatever
        vec![
            (PROGRAMTYPE, "Programs"), 
            (RESOURCETYPE, "Resources")
        ].into_iter().collect()
    }
}

pub enum SubmissionOrder { }

impl SubmissionOrder {
    pub fn list() -> Vec<(&'static str, &'static str)> {
        vec![
            (POPSCORE1SORT, "Popular"), 
            ("id_desc", "Created (newest)"), 
            ("id", "Created (oldest)"),
            ("lastRevisionId_desc", "Edited (newest)"),
            ("lastRevisionId", "Edited (oldest)"),
            ("name", "Alphabetical (A-Z)"),
            ("name_desc", "Alphabetical (Z-A)"),
            ("random", "Random")
        ].into_iter().collect()
    }
}

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
            subtype: Some(String::from(PROGRAMTYPE)), 
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
    add_value!(request, "forcontent", FORCONTENTKEY);

    let mut query = String::from("contentType = @type and !notdeleted()"); 
    // !valuekeynotlike({{system}}) and !notdeleted()";

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
        if subtype == PROGRAMTYPE {
            //MUST have a key unless the user specifies otherwise
            if !search.removed {
                add_value!(request, "dlkeylist", vec![DOWNLOADKEYKEY]);
                query.push_str(" and !valuekeyin(@dlkeylist)");
            }

            if search.system != ANYSYSTEM {
                add_value!(request, "systemkey", SYSTEMSKEY);
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

    add_value!(request, "categorytype", CATEGORYTYPE);
    //add_value!(request, "subtypesearch", format!("%{}%", &search.subtype));
    let mut category_request = build_request!(
        RequestType::content,
        String::from("id,literalType,contentType,values,name"),
        String::from("contentType = @systemtype and !notdeleted() and literalType = @categorytype") // and !valuelike(@forcontent,@subtypesearch)")
    );
    category_request.name = Some(String::from("categories"));
    request.requests.push(category_request);

    request
}

pub fn pageicon(config: &LinkConfig, page: &Content) -> Markup {
    let values = match &page.values {
        Some(values) => values.clone(),
        None => HashMap::new()
    };
    //Is this really inefficient, to continuously make hashes? hopefully not!
    let systems_map = SubmissionSystem::list().into_iter().collect::<HashMap<&str, &str>>();
    html! {
        //Don't forget the program type! if it exists anyway
        @if let Some(systems) = values.get(SYSTEMSKEY).and_then(|k| k.as_array()) {
            @for system in systems {
                @if let Some(system) = system.as_str() {
                    @if let Some(title) = systems_map.get(system) {
                        img title=(title) src={(config.resource_root)"/"(system)".svg"};
                    }
                }
            }
        }
        @else {
            //This must be a resource!
            img title="Resource" src={(config.resource_root)"/sb-page.png"};
        }
    }
}

pub fn page_card(config: &LinkConfig, page: &Content, users: &HashMap<i64, User>) -> Markup {
    let user = user_or_default(users.get(&page.createUserId.unwrap_or(0)));
    //very wasteful allocations but whatever
    let link = forum_thread_link(config, page);
    let values = match &page.values { Some(values) => values.clone(), None => HashMap::new() };
    html!{
        div.{"pagecard "(s(&page.literalType))} {
            div."cardmain" {
                div."cardtext" {
                    a."flatlink" href=(link) { h3 { (s(&page.name)) } }
                    div."description" { (s(&page.description)) }
                }
                //Conditionally render the "cardimage" container
                @if let Some(images) = values.get(IMAGESKEY).and_then(|k| k.as_array()) {
                    //we now have the images: we just need the first one (it's a hash?)
                    @if let Some(image) = images.get(0).and_then(|i| i.as_str()) {
                        a."cardimage" href=(link) {
                            img src=(image_link(config, image, 200, false));
                        }
                    }
                }
            }
            div."smallseparate cardbottom" {
                a."user flatlink" href=(user_link(config, &user)) { (user.username) }
                //This may have conditional display? I don't know, depends on how much room there is!
                time."aside" datetime=(d(&page.createDate)) { (timeago_o(&page.createDate)) } 
                //div."keyspec smallseparate" {
                    @if let Some(key) = values.get(DOWNLOADKEYKEY).and_then(|k| k.as_str()) {
                        span."key" { (key) }
                    }
                    @else if s(&page.literalType) == PROGRAMTYPE {
                        span."key error" { "REMOVED" }
                    }
                    @else {
                        span."key" { /* nothing! just a placeholder! */ }
                    }
                    div."systems" {
                        (pageicon(config, page))
                    }
                //}
            }
        }
    }
}
