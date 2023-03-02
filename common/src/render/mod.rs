pub mod layout;
pub mod forum;
pub mod submissions;

use chrono::*;

use super::*;

// ----------------------------
// *     FORMAT FUNCTIONS     *
// ----------------------------

pub fn timeago_future(time: &chrono::DateTime<chrono::Utc>) -> String {
    let duration = time.signed_duration_since(chrono::Utc::now());
    match duration.to_std() {
        Ok(stdur) => {
            timeago::format(stdur, timeago::Style::HUMAN).replace(" ago", "")
        },
        Err(error) => {
            format!("PARSE-ERR({}):{}", duration, error)
        }
    }
}

pub fn timeago(time: &chrono::DateTime<chrono::Utc>) -> String {
    let duration = chrono::Utc::now().signed_duration_since(*time);
    match duration.to_std() {
        Ok(stdur) => {
            timeago::format(stdur, timeago::Style::HUMAN)
        },
        Err(error) => {
            format!("PARSE-ERR({}):{}", duration, error)
        }
    }
}

pub fn timeago_o(time: &Option<chrono::DateTime<chrono::Utc>>) -> String {
    if let Some(time) = time {
        timeago(time)
    }
    else {
        String::from("???")
    }
}

pub fn b(boolean: bool) -> &'static str {
    if boolean { "true" }
    else { "false" }
}

pub fn d(date: &Option<DateTime<Utc>>) -> String {
    if let Some(date) = date { dd(date) }
    else { String::from("NODATE") }
}

pub fn dd(date: &DateTime<Utc>) -> String {
    date.to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub fn i(int: &Option<i64>) -> String {
    if let Some(int) = int { format!("{}", int) }
    else { String::from("??") }
}


// ------------------
// - SPECIAL FORMAT -
// ------------------

pub const SHORTDESCRIPTION : usize = 200;

pub fn short_post(message: &Message) -> String {
    if let Some(ref text) = message.text {
        text.chars().take(SHORTDESCRIPTION).collect::<String>()
    }
    else {
        String::from("")
    }
}

pub fn short_description(thread: &Content) -> String {
    if let Some(ref description) = thread.description {
        if !description.is_empty() {
            return description.clone();
        }
    }
    if thread.literalType.as_deref() != Some(constants::SBSPageType::FORUMTHREAD) {
        //Get some short portion of the body, even if it's bad? We'll fix bbcode stuff later
        if let Some(ref text) = thread.text {
            return text.chars().take(SHORTDESCRIPTION).collect::<String>();
        }
    }
    return String::from("");
}

pub fn short_description_opt(thread: Option<&Content>) -> String {
    if let Some(thread) = thread {
        short_description(thread)
    }
    else {
        String::from("")
    }
}

// ---------------------
// *    FRAGMENTS      *
// ---------------------

impl LinkConfig 
{
    pub fn style(&self, link: &str) -> Markup {
        html! {
            link rel="stylesheet" href={(self.static_root) (link) "?" (self.cache_bust) } { }
        }
    }

    pub fn script(&self, link: &str) -> Markup {
        html! {
            script src={(self.static_root) (link) "?" (self.cache_bust) } defer { }
        }
    }

    // Produce some metadata for the header that any page can use (even widgets)
    pub fn basic_meta(&self) -> Markup{
        html! {
            //Can I have comments in html markup?
            meta charset="UTF-8";
            meta name="rating" content="general";
            meta name="viewport" content="width=device-width";
            //[] is for optional, {} is for concatenate values
            link rel="icon" type="image/svg+xml" sizes="any" href={(self.resource_root) "/favicon.svg"};
        } 
    }
}

pub fn errorlist(errors: Option<Vec<String>>) -> Markup {
    html! {
        div."errorlist" {
            @if let Some(errors) = errors {
                @for error in errors {
                    div."error" {(error)}
                }
            }
        }
    }
}

