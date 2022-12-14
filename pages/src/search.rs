use super::*;

//use serde_json::Value;

// Eventually move this somewhere else?
//static PROGRAMTYPE: &str = "program";
//static RESOURCETYPE: &str = "resource";

pub fn render(data: MainLayoutData, pages: Vec<Content>, users: HashMap<i64, User>) -> String {
    layout(&data, html!{
        section {

        }
        // All the pages (directly in the section?)
        section."results" {
            div {
                //Or maybe in here
            }
            //Generic pagelist generation (just need data)
            div."smallseparate pagelist" {

            }
            //h1 { "Browse is search"}
            //p { "Search may be simultaneously more powerful and less powerful than before"}
        }
    }).into_string()
}

pub fn page_card(config: &LinkConfig, page: &Content, users: &HashMap<i64, User>) -> Markup {
    let user = user_or_default(users.get(&page.createUserId.unwrap_or(0)));
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
            div."cardbottom" {
                a."user plainlink" href=(user_link(config, &user)) { (user.username) }
                @if let Some(key) = values.get("key").and_then(|k| k.as_str()) {
                    span."key" { (key) }
                }
                //Don't forget the program type!
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum SubmissionType {
    Program,
    Resource
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug)]
pub enum SubmissionOrder {
    id_desc,
    id,
    lastRevisionId_desc,
    lastRevisionId,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Search {
    pub search: Option<String>,
    pub order: SubmissionOrder,
    pub subtype: SubmissionType,
    pub page: i32
}

impl Default for Search {
    fn default() -> Self {
        Self {
            search: None,
            order: SubmissionOrder::id_desc, //Some(String::from("id_desc")), //Inverse create time
            subtype: SubmissionType::Program,   //Show programs first!
            page: 0
        }
    }
}

pub async fn get_render(context: PageContext, search: HashMap<String, String>) -> Result<Response, Error> 
{
    //Manually parse the search, because of the tag magic (no javascript)
    Err(Error::Other(String::from("wow")))
}