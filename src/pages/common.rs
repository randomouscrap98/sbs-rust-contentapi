pub mod contentapi;
pub mod render;

use serde::*;

// These aren't contentapi constants, they're general contsants
pub const DEFAULTPAGESORT: &str = "upvotes";
pub const ANYSYSTEM: &str = "any";
pub const PTCSYSTEM: &str = "ptc";
pub const MARKUPBBCODE: &str = "bbcode";

pub const SBSSYSTEMS: &[(&str, &str)] = &[
    (ANYSYSTEM, "Any"),
    (PTCSYSTEM, "Petit Computer (DSi)"),
    ("3ds", "Nintendo 3DS"),
    ("wiiu", "Nintendo WiiU"),
    ("switch", "Nintendo Switch"),
];

pub fn get_sbs_system_title(key: &str) -> Option<&str> {
    for (sys, title) in SBSSYSTEMS {
        if *sys == key {
            return Some(title);
        }
    }
    return None;
}

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
