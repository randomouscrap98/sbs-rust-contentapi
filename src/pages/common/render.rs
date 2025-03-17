use chrono::{DateTime, SecondsFormat, Utc};
use maud::*;
use std::collections::HashMap;

use super::contentapi::*;
use super::*;
use crate::links::*;

pub fn timeago_future(time: &chrono::DateTime<chrono::Utc>) -> String {
    let duration = time.signed_duration_since(chrono::Utc::now());
    match duration.to_std() {
        Ok(stdur) => timeago::format(stdur, timeago::Style::HUMAN).replace(" ago", ""),
        Err(error) => {
            format!("PARSE-ERR({}):{}", duration, error)
        }
    }
}

pub fn timeago(time: &chrono::DateTime<chrono::Utc>) -> String {
    let duration = chrono::Utc::now().signed_duration_since(*time);
    match duration.to_std() {
        Ok(stdur) => timeago::format(stdur, timeago::Style::HUMAN),
        Err(error) => {
            format!("PARSE-ERR({}):{}", duration, error)
        }
    }
}

pub fn timeago_o(time: &Option<chrono::DateTime<chrono::Utc>>) -> String {
    if let Some(time) = time {
        timeago(time)
    } else {
        String::from("???")
    }
}

pub fn b(boolean: bool) -> &'static str {
    if boolean {
        "true"
    } else {
        "false"
    }
}

pub fn d(date: &Option<DateTime<Utc>>) -> String {
    if let Some(date) = date {
        dd(date)
    } else {
        String::from("NODATE")
    }
}

pub fn dd(date: &DateTime<Utc>) -> String {
    date.to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub fn i(int: &Option<i64>) -> String {
    if let Some(int) = int {
        format!("{}", int)
    } else {
        String::from("??")
    }
}

pub fn pageicon2(
    links: &LinkConfig,
    values: &HashMap<String, String>,
    literal_type: &str,
    max_icons: i32,
) -> Markup {
    let systems = get_systems2(values);
    let mut count = 0;
    html! {
        //Don't forget the program type! if it exists anyway
        @if systems.len() > 0 {
            @for system in systems {
                @if let Some(title) = get_sbs_system_title(&system) {
                    img title=(title) class="sysicon" src={(links.resource_root)"/"(system)".svg"};
                    ({
                        count = count + 1;
                        if count >= max_icons { break; }
                        ""
                    })
                }
            }
        }
        @else {
            @if literal_type == SBSPageType::DOCUMENTATION {
                //This icon may change later
                img title="Documentation" src={(links.resource_root)"/sb-docs.png"};
            }
            @else if literal_type == SBSPageType::RESOURCE {
                img title="Resource" src={(links.resource_root)"/sb-info.png"};
            }
            @else {
                //Anything else
                img title="Thread" src={(links.resource_root)"/sb-page.png"};
            }
        }
    }
}

pub fn page_card2(links: &LinkConfig, page: &BrowseContent, users: &HashMap<i64, User2>) -> Markup {
    let user = user_or_default2(users.get(&page.create_user_id));
    //very wasteful allocations but whatever
    let link = links.forum_thread_unsafe(&page.hash);
    let values = &page.values;
    let mut image: Option<String> = None;
    if let Some(images_raw) = values.get(SBSValue::IMAGES) {
        match serde_json::from_str::<Vec<String>>(images_raw) {
            Ok(images) => {
                if let Some(first_image) = images.get(0) {
                    image = Some(first_image.clone());
                }
            }
            Err(e) => {
                println!("WARN: Couldn't parse images json! {}", e);
            }
        }
    }
    let mut key: Option<String> = None;
    if let Some(keyvalue) = values.get(SBSValue::DOWNLOADKEY) {
        match serde_json::from_str::<String>(keyvalue) {
            Ok(rawkey) => {
                key = Some(rawkey.clone());
            }
            Err(e) => {
                println!("WARN: Couldn't parse key json! {}", e);
            }
        }
    }
    // match &page.values {
    //     Some(values) => values.clone(),
    //     None => HashMap::new(),
    // };
    let systems = get_systems2(values);
    html! {
        div.{"pagecard "(page.literal_type)} {
            div."cardmain" {
                div."cardtext" {
                    a."flatlink" href=(link) { h3 { (page.name) } }
                    div."description" { (page.description) }
                }
                //Conditionally render the "cardimage" container
                @if let Some(image) = &image {
                    a."cardimage" href=(link) {
                        img src=(links.image(image, QueryImage::Scaled300));
                    }
                }
            }
            div."smallseparate cardbottom" {
                a."user flatlink" href=(links.user_unsafe(&user.username)) { (user.username) }
                //This may have conditional display? I don't know, depends on how much room there is!
                time."aside" datetime=(dd(&page.create_date)) { (timeago(&page.create_date)) }
                //All this junk needs "key" so it can display properly... probably should change this?
                @if let Some(key) = key {
                    span."key" { (key) }
                }
                @else if systems.iter().any(|s| s == &PTCSYSTEM) {
                    a."key" href=(links.qr_generator_unsafe(&page.hash)) { "QR Codes" }
                }
                @else if page.literal_type == SBSPageType::PROGRAM {
                    span."key error" { "REMOVED" }
                }
                @else {
                    span."key" { /* nothing! just a placeholder! */ }
                }
                div."systems" {
                    (pageicon2(links, values, &page.literal_type, 99))
                }
            }
        }
    }
}
