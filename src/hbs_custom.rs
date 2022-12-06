use chrono::{DateTime, Utc};
use rocket_dyn_templates::handlebars::{self, Handlebars};
use serde::Serialize;
use lazy_static::lazy_static;
use crate::api_data;
use crate::bbcode::{BBCode, TagInfo, TagType};
use serde_qs;

//I think I'm doing something wrong? I don't like that I need all these
static FILERAWKEY: &'static str = "api_fileraw";
static HTTPROOTKEY: &'static str = "http_root";
static HTTPSTATICKEY: &'static str = "http_static";
static BOOTTIMEKEY: &'static str = "boot_time";
static ROUTEKEY: &'static str = "route_path";

//This is the one place we have our bbcode rendering context. It'll take up a little
//memory to store it (maybe a couple kb?) but it keeps us from hvaing to recompile regex.
lazy_static! {
    static ref BBCODE : BBCode = {
        let mut matchers = BBCode::basics().unwrap(); //this better not fail! It'll fail very early though
        let mut my_tags = BBCode::tags_to_matches(vec![
            TagInfo { tag: "quote", outtag: "blockquote", tag_type : TagType::DefinedArg("cite"), rawextra : None, force_verbatim: false  },
            TagInfo { tag: "anchor", outtag: "a", tag_type : TagType::DefinedArg("name"), rawextra : None, force_verbatim: true  },
            TagInfo { tag: "icode", outtag: "span", tag_type : TagType::Simple, rawextra : Some(r#"class="icode""#), force_verbatim: true  },
            TagInfo { tag: "code", outtag: "pre", tag_type : TagType::Simple, rawextra : Some(r#"class="code" data-code"#), force_verbatim: true  },
            TagInfo { tag: "youtube", outtag: "a", tag_type : TagType::DefaultArg("href"), rawextra : Some(r#"class="youtube" data-youtube"#), force_verbatim: true  },
            TagInfo { tag: "spoiler", outtag: "span", tag_type : TagType::DefinedArg("data-title"), rawextra : Some(r#"class="spoilertext" data-spoiler"#), force_verbatim: false },
            TagInfo::simple("h1"),
            TagInfo::simple("h2"),
            TagInfo::simple("h3"),
        ]).unwrap();
        matchers.append(&mut my_tags);
        BBCode { matchers } 
    };
}

//The helper signature is just TOO DAMN COMPLICATED (I know you need those
//params sometimes) so I'm just... simplifying it
macro_rules! generate_helper {
    ($name:ident, $h:ident, $out:ident, $ctx:ident, $code:block) => {
        fn $name(
            $h: &handlebars::Helper<'_, '_>,
            _: &handlebars::Handlebars,
            $ctx : &handlebars::Context,
            _: &mut handlebars::RenderContext<'_, '_>,
            $out: &mut dyn handlebars::Output
        ) -> handlebars::HelperResult {
            $code
            Ok(())
        }
    };
}

macro_rules! get_param {
    ($h:ident, $i:literal, $t:ident) => {
       $h.param($i).and_then(|v| v.value().$t()) 
    };
}

macro_rules! get_data {
    ($ctx:ident, $key:expr, $t:ident) => {
        $ctx.data().get($key).and_then(|v| v.$t())
    };
}

macro_rules! get_required_str {
    ( ($key:ident, $name:ident, $ctx:ident) $block:block) => {
        if let Some($name) = get_data!($ctx, $key, as_str) {
            $block
        }
        else {
            //One day may return actual error
            println!("No {} in context! Not rendering anything!", $key)
        }
    };
}

generate_helper!{imagelink_helper, h, out, ctx, {
    //The minimum viable is the hash. Next is size and then cropping
    if let Some(hash) = get_param!(h, 0, as_str) {
        get_required_str! { (FILERAWKEY, fileroot, ctx) {
            let query = api_data::QueryImage {
                size: get_param!(h, 1, as_i64),
                crop: get_param!(h, 2, as_bool)
            };
            //println!("Query struct: {:?}", query);
            //println!("H params: {:?}", h.params());
            match serde_qs::to_string(&query) {
                Ok(querystring) => {
                    let link = format!("{}/{}?{}", fileroot, hash, querystring);
                    out.write(&link)?;
                },
                Err(error) => println!("Serde_qs failed? Not printing link for {}. Error: {}", hash, error)
            }
        }}
    }
}}

//Generate certain attributes in the header link, mainly href and class. Header links are styled special
//if they're the selected tab
generate_helper!{headerlink_helper, h, out, ctx, {
    //Absolutely must have the path
    if let Some(path) = get_param!(h, 0, as_str) {
        //Have some if/else paths, we do auto-print errors for required rather than throwing. But
        //one day, might throw instead of printing.
        get_required_str! { (HTTPROOTKEY, httproot, ctx) {
            get_required_str! { (ROUTEKEY, route, ctx) {
                out.write(&format!("href=\"{}{}\"", httproot, path))?;
                out.write("class=\"plainlink headertab ")?; //Had "hover" here at one point, not sure about it...
                //THIS is the point of this helper! Add a special class if this is the current page!
                if route.starts_with(path) {
                    out.write("current")?;
                }
                out.write("\"")?;
            }}
        }}
    }
}}

//Generate the entire style element for the given path (single parameter, auto cache busting)
generate_helper!{stylesheet_helper, h, out, ctx, {
    //Absolutely must have the path to the css file (relative to static!)
    if let Some(path) = get_param!(h, 0, as_str) {
        get_required_str! { (HTTPSTATICKEY, httpstatic, ctx) {
            get_required_str! { (BOOTTIMEKEY, boot_time, ctx) {
                out.write(&format!("<link rel=\"stylesheet\" href=\"{}{}?{}\">", httpstatic, path, boot_time))?;
            }}
        }}
    }
}}

//Same as stylesheet helper except output different format
generate_helper!{script_helper, h, out, ctx, {
    //Absolutely must have the path to the css file (relative to static!)
    if let Some(path) = get_param!(h, 0, as_str) {
        get_required_str! { (HTTPSTATICKEY, httpstatic, ctx) {
            get_required_str! { (BOOTTIMEKEY, boot_time, ctx) {
                out.write(&format!("<script src=\"{}{}?{}\"></script>", httpstatic, path, boot_time))?;
            }}
        }}
    }
}}

generate_helper!{selfpost_helper, h, out, ctx, {
    get_required_str! { (HTTPROOTKEY, httproot, ctx) {
        get_required_str! { (ROUTEKEY, route, ctx) {
            out.write(&format!("method=\"POST\" action=\"{httproot}{route}"))?;
            //This lets us select different routes without actually changing the path in the url,
            //meaning users won't get confused (the url should now always be correct)
            if let Some(classify) = get_param!(h, 0, as_str) {
                out.write("?")?;
                out.write(classify)?;
            }
            out.write("\"")?;
        }}
    }}
}}

generate_helper!{string_helper, h, out, _ctx, {
    if let Some(int) = get_param!(h, 0, as_i64) {
        out.write(&format!("{}",int))?;
    }
}}

generate_helper!{bbcode_helper, h, out, _ctx, {
    if let Some(text) = get_param!(h, 0, as_str) {
        out.write(&BBCODE.parse(text))?;
    }
}}

generate_helper!{timeago_helper, h, out, _ctx, {
    if let Some(time) = get_param!(h, 0, as_str) {
        match DateTime::parse_from_rfc3339(time) {
            Ok(ptime) => {
                let duration = Utc::now().signed_duration_since(ptime); //timeago::format()
                match duration.to_std() {
                    Ok(stdur) => {
                        let agotime = timeago::format(stdur, timeago::Style::HUMAN);
                        out.write(&agotime)?;
                    },
                    Err(error) => {
                        println!("Couldn't convert chrono duration {} to std: {}", duration, error);
                    }
                }
            }
            Err(error) => {
                println!("Couldn't parse date {}: {}", time, error);
            }
        }
    }
}}

//A stupid extension to help display of form select, it's like impossible to 
//set the selected option, so this will go along with the partial
#[derive(Serialize, FromForm, Debug)]
pub struct SelectValue<T>
{
    pub value: T,
    pub text: Option<String>,
    pub selected: bool
}

impl<T:PartialEq+Copy> SelectValue<T> {
    pub fn new(value: T, text: &str, selected_value: T) -> Self {
        SelectValue { 
            value: value,
            selected: selected_value == value, 
            text: Some(String::from(text))
        }
    }
}


// Where we register all the helpers
pub fn customize(hbs: &mut Handlebars) {
    hbs.register_helper("imagelink", Box::new(imagelink_helper));
    hbs.register_helper("headerlink", Box::new(headerlink_helper));
    hbs.register_helper("selfpost", Box::new(selfpost_helper));
    hbs.register_helper("stylesheet", Box::new(stylesheet_helper));
    hbs.register_helper("script", Box::new(script_helper));
    hbs.register_helper("timeago", Box::new(timeago_helper));
    hbs.register_helper("string", Box::new(string_helper));
    hbs.register_helper("bbcode", Box::new(bbcode_helper));
}
