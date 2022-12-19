use contentapi::{*, conversion::map_users};

use super::*;
use common::page::*;
use common::layout::*;

//use serde_json::Value;

// Eventually move this somewhere else?
//static PROGRAMTYPE: &str = "program";
//static RESOURCETYPE: &str = "resource";
pub fn render(data: MainLayoutData, pages: Vec<Content>, users: HashMap<i64, User>, search: Search,
    categories: Vec<Category>) -> String 
{
    //Need to split category search into parts 
    //let search_system = match &search.system { Some(system) => system, None => };
    layout(&data, html!{
        (style(&data.config, "/forpage/search.css"))
        (script(&data.config, "/forpage/search.js"))
        section {
            //Don't include an action so it just posts to the same url but with the form as params...?
            form."smallseparate" method="GET" id="searchform" {
                label."inline" for="search-type" {
                    span{"Type: "}
                    select #"search-type" name="subtype" {
                        @for (value,text) in SubmissionType::list() {
                            option value=(value) selected[value == search.subtype] { (text) }
                        }
                    }
                }
                //THIS needs to come from parameters! Don't know the categories available unless
                //we look at the database!
                label."inline" for="search-category" 
                {
                    span{"Category:"}
                    select #"search-category" name="category" {
                        option value="0" { "Any" }
                        @for category in &categories {
                            option data-for=(category.forcontent) value=(category.id) selected[Some(category.id) == search.category] { 
                                (category.name) 
                            }
                        }
                    }
                }
                @if search.subtype == PROGRAMTYPE {
                    label."inline" for="search-system" {
                        span{"System: "}
                        select #"search-system" name="system" {
                            @for (value,text) in SubmissionSystem::list() {
                                option value=(value) selected[value == search.system] { (text) }
                            }
                        }
                    }
                }
                label."inline" for="search-order" {
                    span{"Order: "}
                    select #"search-order" name="order" {
                        @for (value,text) in SubmissionOrder::list() {
                            option value=(value) selected[value == search.order] { (text) }
                        }
                    }
                }
                label."inline" for="search-text" {
                    span { "Search: " }
                    input."" #"search-text" type="text" name="search" value=[&search.search];
                }
                @if search.subtype == PROGRAMTYPE {
                    label."inline" for="search-removed" {
                        span { "Show removed: " }
                        input."" #"search-text" type="checkbox" name="removed" checked[search.removed] value="true";
                    }
                }
                label."inline" for="search-page" {
                    span {"Page: "}
                    input."smallinput" #"search-page" type="text" name="page" value=(search.page); 
                }

                input type="submit" value="Update search";
            }
        }
        // All the pages (directly in the section?)
        section."results" {
            div."resultslist" {
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


#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Search {
    pub search: Option<String>,
    pub order: String, //SubmissionOrder,
    pub subtype: String, //SubmissionType,
    //pub system: Option<String>,
    pub system: String,
    pub category: Option<i64>,
    pub removed: bool,
    pub page: i32
}

impl Default for Search {
    fn default() -> Self {
        Self {
            search: None,
            order: String::from(POPSCORE1SORT), //SubmissionOrder::id_desc, //Some(String::from("id_desc")), //Inverse create time
            subtype: String::from(PROGRAMTYPE), ////SubmissionType::Program,   //Show programs first!
            system: String::from(ANYSYSTEM),
            //system: None,
            category: None,
            removed: false, //By default, DON'T show removed!
            page: 0
        }
    }
}

pub struct Category {
    pub id: i64,
    pub name: String,
    pub forcontent: String
}

pub async fn get_render(context: PageContext, search: Search, per_page: i32) -> Result<Response, Error> 
{
    //Build up the request based on the search, then render
    let mut request = FullRequest::new();
    add_value!(request, "type", ContentType::PAGE);
    add_value!(request, "systemtype", ContentType::SYSTEM);
    add_value!(request, "subtype", search.subtype.clone());
    add_value!(request, "forcontent", FORCONTENTKEY);

    let mut query = String::from("contentType = @type and !notdeleted() and literalType = @subtype"); 
    // !valuekeynotlike({{system}}) and !notdeleted()";

    if let Some(stext) = &search.search {
        add_value!(request, "text", format!("%{}%", stext));
        query.push_str(" and (name like @text or !keywordlike(@text))");
    }

    if let Some(category) = &search.category {
        if *category != 0 {
            add_value!(request, "categoryTag", vec![format!("tag:{}", category)]);
            query.push_str(" and !valuekeyin(@categoryTag)");
        }
    }

    //Ignore certain search criteria
    if search.subtype == PROGRAMTYPE {
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

    //let fields = "id,hash,contentType,createUserId";
    //let order = String::from(if search.oldest { "id" } else { "id_desc" });
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

    let result = context.api_context.post_request(&request).await?;
    //println!("RESULT: {:#?}", &result);
    let pages = conversion::cast_result_safe::<Content>(&result, "content")?;
    let users = conversion::cast_result_safe::<User>(&result, "user")?;
    let categories = conversion::cast_result_safe::<Content>(&result, "categories")?;
    let users = map_users(users);

    let categories = categories.into_iter().map(|c| {
        Category {
            id: c.id.unwrap_or(0),
            name: c.name.unwrap_or_else(|| String::from("")), //Only evaluated on failure
            forcontent: c.values
                .and_then(|v| v.get(FORCONTENTKEY).and_then(|v2| v2.as_str()).and_then(|v3| Some(String::from(v3))))
                .unwrap_or_else(|| String::from(""))
        }
    }).collect::<Vec<Category>>();

    //Manually parse the search, because of the tag magic (no javascript)
    //Err(Error::Other(String::from("wow")))
    Ok(Response::Render(render(context.layout_data, pages,  users, search, categories)))
}