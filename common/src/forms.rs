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
    pub post: Option<String> //Not present on thread edits
}

//#[derive(Serialize, Deserialize, Debug)]
//pub struct EditThread
//{
//    pub id: i64,
//    pub parent_id: i64,
//    pub title: String,
//    //Not sure what else to do with this for now
//}

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
            subtype: Some(String::from(SBSPageType::PROGRAM)), 
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
