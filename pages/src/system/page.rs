
use super::super::*;

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
        ].into_iter().collect()
    }
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