use rocket_dyn_templates::handlebars::{self, Handlebars};
use crate::api_data;
use serde_qs;

//I think I'm doing something wrong? I don't like that I need all these
static FILERAWKEY: &'static str = "api_fileraw";
static HTTPROOTKEY: &'static str = "http_root";
static ROUTEKEY: &'static str = "route_path";

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
            println!("Query struct: {:?}", query);
            println!("H params: {:?}", h.params());
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
                out.write("class=\"plainlink headertab ")?;
                //THIS is the point of this helper! Add a special class if this is the current page!
                if route.starts_with(path) {
                    out.write("current")?;
                }
                out.write("\"")?;
            }}
        }}
    }
}}

generate_helper!{selfpost_helper, _h, out, ctx, {
    get_required_str! { (HTTPROOTKEY, httproot, ctx) {
        get_required_str! { (ROUTEKEY, route, ctx) {
            out.write(&format!("method=\"POST\" action=\"{httproot}{route}\""))?;
        }}
    }}
}}

// Where we register all the helpers
pub fn customize(hbs: &mut Handlebars) {
    hbs.register_helper("imagelink", Box::new(imagelink_helper));
    hbs.register_helper("headerlink", Box::new(headerlink_helper));
    hbs.register_helper("selfpost", Box::new(selfpost_helper));
}
