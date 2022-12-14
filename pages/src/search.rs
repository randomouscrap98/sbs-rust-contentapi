use super::*;

//use serde_json::Value;

// Eventually move this somewhere else?
//static PROGRAMTYPE: &str = "program";
//static RESOURCETYPE: &str = "resource";
static DOWNLOADKEYKEY: &str = "dlkey";
static SYSTEMSKEY: &str = "systems";

pub fn render(data: MainLayoutData, pages: Vec<Content>, users: HashMap<i64, User>, search: Search) -> String {
    layout(&data, html!{
        section {
            //Don't include an action so it just posts to the same url but with the form as params...?
            form method="GET" id="searchform" {
                label."inline" for="search-text" {
                    span { "Search: " }
                    input."" #"search-text" type="text" name="search" value=[&search.search];
                }
                label."inline" for="search-type" {
                    span{"Type:"}
                    select #"search-type" name="type" {
                        @for (value,text) in SubmissionType::list() {
                            option value=(value) selected[value == search.subtype] { (text) }
                        }
                    }
                }
                //THIS needs to come from parameters! Don't know the categories available unless
                //we look at the database!
                //label."inline" for="search-category" {
                //    span{"Category:"}
                //    select #"search-category" name="category" {
                //        @for (value,text) in Submi::list() {
                //            option value=(value) selected[value == search.category] { (text) }
                //        }
                //    }
                //}
                label."inline" for="search-page" {
                    span {"Page:"}
                    input."smallinput" #"search-page" type="text" name="page" value=(search.page); 
                }

                input type="submit" value="Update search";
            }
        }
        // All the pages (directly in the section?)
        section."results" {
            div {
                //Or maybe in here
                @for page in &pages {
                    (page_card(&data.config, page, &users))
                }
            }
            //Generic pagelist generation (just need data)
            (page_navigation(&data, &search))
            //h1 { "Browse is search"}
            //p { "Search may be simultaneously more powerful and less powerful than before"}
        }
    }).into_string()
}

pub fn page_card(config: &LinkConfig, page: &Content, users: &HashMap<i64, User>) -> Markup {
    let user = user_or_default(users.get(&page.createUserId.unwrap_or(0)));
    //very wasteful allocations but whatever
    let systems_map = SubmissionSystem::list();
    let values = match &page.values {
        Some(values) => values.clone(),
        None => HashMap::new()
    };
    html!{
        a.{"pagecard "(s(&page.literalType))} {
            div."cardmain" {
                h3 { (s(&page.name)) }
                div."description" { (s(&page.description)) }
            }
            div."smallseparate cardbottom" {
                a."user plainlink" href=(user_link(config, &user)) { (user.username) }
                //This may have conditional display? I don't know, depends on how much room there is!
                time."aside" datetime=(d(&page.createDate)) { (timeago_o(&page.createDate)) } 
                @if let Some(key) = values.get(DOWNLOADKEYKEY).and_then(|k| k.as_str()) {
                    span."key" { (key) }
                }
                div."systems" {
                    //Don't forget the program type! if it exists anyway
                    @if let Some(systems) = values.get(SYSTEMSKEY).and_then(|k| k.as_array()) {
                        @for system in systems {
                            @if let Some(system) = system.as_str() {
                                @if let Some(title) = systems_map.get(system) {
                                    img title=(title) src={(config.resource_root)"/"(system)"svg"};
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
        }
    }
}

// TODO: Make this generic across imagebrowse and here? Search has to impl some trait with get/set 
// page functions and clone, and .browsepagenav might need to go in base.css
fn page_navigation(data: &MainLayoutData, search: &Search) -> Markup {
    let mut searchprev = search.clone();
    let mut searchnext = search.clone();
    searchprev.page = searchprev.page - 1;
    searchnext.page = searchnext.page + 1;
    html! {
        div."smallseparate browsepagenav" {
            @if let Ok(prevlink) = serde_urlencoded::to_string(searchprev) {
                a."coolbutton" href={(self_link(&data))"?"(prevlink)} {"Previous"}
            }
            @if let Ok(nextlink) = serde_urlencoded::to_string(searchnext) {
                a."coolbutton" href={(self_link(&data))"?"(nextlink)} {"Next"}
            }
        }
    }
}


pub enum SubmissionSystem { }

impl SubmissionSystem {
    pub fn list() -> HashMap<&'static str, &'static str> {
        //Idk, whatever
        vec![
            ("3ds", "Nintendo 3DS"), 
            ("wiiu", "Nintendo WiiU"), 
            ("switch", "Nintendo Switch")
        ].into_iter().collect()
    }
}

//#[derive(Serialize, Deserialize, Debug)]
//#[serde(rename_all = "camelCase")]
pub enum SubmissionType { }
//    Program,
//    Resource
//}

impl SubmissionType {
    pub fn list() -> HashMap<&'static str, &'static str> {
        //Idk, whatever
        vec![
            ("program", "Programs"), 
            ("resource", "Resources")
        ].into_iter().collect()
    }
}

//#[allow(non_camel_case_types)]
//#[derive(Serialize, Deserialize, Debug)]
pub enum SubmissionOrder { }
//    id_desc,
//    id,
//    lastRevisionId_desc,
//    lastRevisionId,
//    name,
//    name_desc
//}

impl SubmissionOrder {
    pub fn list() -> HashMap<&'static str, &'static str> {
        vec![
            ("id_desc", "Created (newest)"), 
            ("id", "Created (oldest)"),
            ("lastRevisionId_desc", "Edited (newest)"),
            ("lastRevisionId", "Edited (oldest)"),
            ("name", "Alphabetical (A-Z)"),
            ("name_desc", "Alphabetical (Z-A)"),
        ].into_iter().collect()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Search {
    pub search: Option<String>,
    pub order: String, //SubmissionOrder,
    pub subtype: String, //SubmissionType,
    pub category: Option<String>,
    pub page: i32
}

impl Default for Search {
    fn default() -> Self {
        Self {
            search: None,
            order: String::from("id_desc"), //SubmissionOrder::id_desc, //Some(String::from("id_desc")), //Inverse create time
            subtype: String::from("program"), ////SubmissionType::Program,   //Show programs first!
            category: None,
            page: 0
        }
    }
}

pub async fn get_render(context: PageContext, search: Search) -> Result<Response, Error> 
{
    //Build up the request based on the search, then render
    

    //Manually parse the search, because of the tag magic (no javascript)
    Err(Error::Other(String::from("wow")))
}