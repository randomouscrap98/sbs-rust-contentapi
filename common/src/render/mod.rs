pub mod layout;
pub mod forum;
pub mod submissions;

use chrono::*;

use crate::constants::SBSMARKUPS;

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
            link rel="stylesheet" href={(self.static_root) (link) "?" (self.cache_bust) };
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


#[derive(Default)]
pub struct PostTextboxConfig {
    pub textbox_id: Option<String>,
    pub textbox_name: Option<String>,
    pub textbox_value: Option<String>,
    pub textbox_label: Option<String>,
    pub markup_options: Option<Vec<(String,String)>>,
    pub markup_id: Option<String>,
    pub markup_name: Option<String>,
    pub markup_value: Option<String>,
    pub markup_label: Option<String>,
    //pub labels: bool
}

impl PostTextboxConfig {
    /// Create a config for a basic textbox with auto-generated ids and NO markup selector
    pub fn basic(label: Option<&str>, name: &str, value: &str) -> Self {
        let mut result = Self::default();
        result.textbox_label = label.and_then(|l| Some(l.to_string()));
        //Auto generate an id since it probably doesn't matter (we just need it to make the label nice)
        result.textbox_id = Some(random_id("textedit"));
        result.textbox_name = Some(name.to_string());
        result.textbox_value = Some(value.to_string());
        result
    }
    /// Create a config for a basic textbox with auto-generated ids WITH a markup selector (using all available markup)
    pub fn basic_with_markup(label: Option<&str>, name: &str, value: &str, mname: &str, mvalue: Option<&str>) -> Self {
        let mut result = Self::basic(label, name, value);
        result.markup_options = Some(SBSMARKUPS.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect());
        result.markup_id = Some(random_id("markupselect"));
        result.markup_name = Some(mname.to_string());
        result.markup_value = mvalue.and_then(|v| Some(v.to_string())); //Some(mvalue.to_string());
        result.markup_label = Some("Markup:".to_string());
        result
    }
    pub fn tid(mut self, id: &str) -> Self {
        self.textbox_id = Some(id.to_string());
        self
    }
}

//Eventually may expand this
pub fn post_textbox(config: PostTextboxConfig) -> Markup //id: Option<&str>, name: Option<&str>, value: Option<&str>) -> Markup
{
    html! {
        //Put it all under a common parent so we know exactly how to get to all the parts
        div."markupeditor" {
            @if let Some(ref tlabel) = config.textbox_label {
                label for=[&config.textbox_id] { (tlabel) }
            }
            textarea id=[&config.textbox_id] type="text" name=(opt_s!(config.textbox_name)) required 
                placeholder=r##"[b]bold[/b], [i]italic[/i], 
[u]underline[/u], [s]strikethrough[/s], 
[spoiler=text]hidden[/spoiler], [quote=user]text[/quote]
    "##         { (opt_s!(config.textbox_value)) }
            //So we display the markup ONLY IF we get something for the markup options
            @if let Some(ref markups) = config.markup_options {
                //div."inline" {
                    @if let Some(ref mlabel) = config.markup_label {
                        label for=[&config.markup_id] { (mlabel) }
                    }
                    select id=[&config.markup_id] name=(opt_s!(config.markup_name)) required {
                        @for (key, value) in markups {
                            option value=(key) selected[Some(key) == config.markup_value.as_ref()] { (value) }
                        }
                    }
                //}
            }
        }
    }
}