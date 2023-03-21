use serde::{Serialize, Deserialize};

use crate::constants::*;

// ------------------------
// *     GENERIC FORMS    *
// ------------------------

#[derive(Serialize, Deserialize, Debug)]
pub struct EmailGeneric
{
    pub email: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BasicText
{
    pub text: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BanForm
{
    pub user_id: i64,
    pub reason: String,
    pub hours: f64,
    #[serde(default)]
    pub full: bool
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UnbanForm
{
    pub id: i64,
    pub new_reason: String,
}

#[derive(Deserialize, Debug)]
pub struct BasicPage
{
    pub id: i64,
    pub text: String
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ThreadForm
{
    pub id: i64, //Should default to 0
    pub parent_id: i64,
    pub title: String,
    pub keywords: String,
    pub post: Option<String>, //Not present on thread edits

    //An edit field
    pub edit_message: Option<String>
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PostForm
{
    pub id: i64,
    pub content_id: i64,
    pub reply_id: Option<i64>,
    pub post: String, //Always needed on post, of course
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PageForm
{
    pub id: i64, //Should default to 0
    pub subtype: String, //Has to be SOMETHING, and the post endpoint will reject invalid values
    pub title: String,
    pub text: String,
    pub description: String,    //Making this required now
    pub keywords: String,       //List of keywords separated by space, gets split afterwards

    //These are optional; required for pages + resources but not for documentation
    pub categories: Option<String>,     //Same as keywords
    pub images: Option<String>,         

    //These are optional fields, for programs
    pub key: Option<String>,
    pub version: Option<String>,
    pub size: Option<String>,
    pub systems: Option<String>,     //Same as keywords

    /// The special ptc field. This requires some js systems to construct an appropriate string,
    /// the format of which is understood by the rust frontend to generate qr codes on the fly
    pub ptc_files: Option<String>,

    //Documentation fields
    pub docpath: Option<String>,
    pub markup: Option<String>, //May eventually be more than just documentation;
    pub hash: Option<String>,

    //Edit fields
    pub edit_message: Option<String>
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct VoteForm
{
    pub vote: String
}

// ------------------------
// *    QUERY PARAMS      *
// ------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct PageSearch {
    pub search: Option<String>,
    pub order: String, 
    pub subtype: Option<String>, 
    pub system: String,
    pub category: Option<i64>,
    pub user_id: Option<i64>,
    pub removed: bool,
    pub page: i32
}

impl Default for PageSearch {
    fn default() -> Self {
        Self {
            search: None,
            order: String::from(POPSCORE1SORT), 
            subtype: None, //Some(String::from(SBSPageType::PROGRAM)), 
            system: String::from(ANYSYSTEM),
            user_id: None,
            category: None,
            removed: false, //By default, DON'T show removed!
            page: 0
        }
    }
}

// Unfortunately need this in here so the post knows how to render the iframe
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ThreadQuery {
    pub reply: Option<i64>,
    pub selected: Option<i64>
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AdminSearchParams {
    pub banpage: u32,
    pub logpage: u32,
    pub bans_only: bool
}

#[derive(Deserialize, Debug)]
pub struct UserUpdate
{
    pub username: String,
    pub avatar: String
}
