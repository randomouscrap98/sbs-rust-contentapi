use serde::{Deserialize, Serialize};

use crate::constants::*;

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
    pub page: i32,
}

impl Default for PageSearch {
    fn default() -> Self {
        Self {
            search: None,
            order: String::from(DEFAULTPAGESORT),
            subtype: None,
            system: String::from(ANYSYSTEM),
            user_id: None,
            category: None,
            removed: false, //By default, DON'T show removed!
            page: 0,
        }
    }
}

// Unfortunately need this in here so the post knows how to render the iframe
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ThreadQuery {
    pub reply: Option<i64>,
    pub selected: Option<i64>,
}
