#[macro_export]
macro_rules! string_const {
    ($name:ident => {
        $(($item:ident:$val:literal)),*$(,)?
    }) => {
        #[allow(dead_code)] //man, idk if i'll use ALL of them but I WANT them
        pub enum $name { }

        impl $name {
        $(
            pub const $item: &str = $val;    
        )*
        }
    };
}

pub const CONTENT_CHEAPFIELDS : &str = "~keywordCount,popScore1,lastRevisionId,watchCount,commentCount,lastCommentId,engagement,keywords,permissions";

string_const!{ SBSValue => {
    (DOWNLOADKEY:"dlkey"),
    (VERSION:"version"),
    (SIZE:"size"),
    (SYSTEMS:"systems"),
    (IMAGES:"images"),
    (FORCONTENT:"forcontent")
}}

string_const!{ SBSPageType => {
    (PROGRAM:"program"),
    (RESOURCE:"resource"),
    (CATEGORY:"category"),
    (FORUMCATEGORY:"forumcategory"),
    (FORUMTHREAD:"forumthread"),
    (DIRECTMESSAGES:"directmessages"),
    (DIRECTMESSAGE:"directmessage"),
    (ALERT:"alert"),
    (FRONTPAGE:"frontpage"),
    (SUBMISSIONS:"submissions"),
    (PTCFILES:"ptcfiles"),
    (DOCPARENT:"docparent"),
    (DOCUMENTATION:"documentation")
}}


pub const USERTHEMES: &[(&str,&str)] = &[
    ("sbs", "SBS (default)"),
    ("sbs-dark", "SBS Dark"),
    ("sbs-blue", "SBS Blue"),
    ("sbs-contrast", "SBS High Contrast"),
    ("sbs-dark-contrast", "SBS Dark High Contrast")
];

pub const UPVOTE: &str = "+";
pub const DOWNVOTE: &str = "-";
pub const VOTETYPE: &str = "vote";

pub const POPSCORE1SORT: &str = "popScore1_desc";
pub const ANYSYSTEM: &str = "any";
pub const PTCSYSTEM: &str = "ptc";

pub const SBSSYSTEMS: &[(&str,&str)] = &[
    (ANYSYSTEM, "Any"), 
    (PTCSYSTEM, "Petit Computer (DSi)"), 
    ("3ds", "Nintendo 3DS"), 
    ("wiiu", "Nintendo WiiU"), 
    ("switch", "Nintendo Switch")
]; 

pub fn get_sbs_system_title(key: &str) -> Option<&str> {
    for (sys, title) in SBSSYSTEMS {
        if *sys == key {
            return Some(title);
        }
    }
    return None;
}

pub const ACTIVITYTYPES : &[&str] = &[
    SBSPageType::PROGRAM, 
    SBSPageType::RESOURCE,
    SBSPageType::FORUMTHREAD,
    SBSPageType::DOCUMENTATION
];

pub const THREADTYPES : &[&str] = &[
    SBSPageType::FORUMTHREAD,
    SBSPageType::PROGRAM,
    SBSPageType::RESOURCE,
    SBSPageType::DIRECTMESSAGE,
    SBSPageType::DOCUMENTATION
];

pub const FORUMCATEGORYTYPES : &[&str] = &[
    SBSPageType::FORUMCATEGORY,
    SBSPageType::SUBMISSIONS,
    SBSPageType::DOCPARENT
];

pub const SEARCHPAGETYPES: &[(&str,&str)] = &[
    ("", "Any"),
    (SBSPageType::PROGRAM, "Programs"), 
    (SBSPageType::RESOURCE, "Resources")
];

pub const SEARCHPAGEORDERS: &[(&str,&str)] = &[
    (POPSCORE1SORT, "Popular"), 
    ("id_desc", "Created (newest)"), 
    ("id", "Created (oldest)"),
    ("lastRevisionId_desc", "Edited (newest)"),
    ("lastRevisionId", "Edited (oldest)"),
    ("name", "Alphabetical (A-Z)"),
    ("name_desc", "Alphabetical (Z-A)"),
    ("random", "Random")
];

pub const CATEGORYPREFIX: &str = "tag:";