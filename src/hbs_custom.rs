use rocket_dyn_templates::handlebars::{self, Handlebars};
use crate::api_data;
use serde_qs;

static FILERAWKEY: &'static str = "api_fileraw";

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

generate_helper!{imagelink_helper, h, out, ctx, {
    //The minimum viable is the hash. Next is size and then cropping
    if let Some(hash) = get_param!(h, 0, as_str) {
        if let Some(fileroot) = get_data!(ctx, FILERAWKEY, as_str) {
            let query = api_data::QueryImage {
                hash: String::from(hash), 
                size: get_param!(h, 1, as_i64),
                crop: get_param!(h, 2, as_bool)
            };
            match serde_qs::to_string(&query) {
                Ok(querystring) => {
                    let link = format!("{}/{}?{}", fileroot, hash, querystring);
                    out.write(&link)?;
                },
                Err(error) => println!("Serde_qs failed? Not printing link for {}. Error: {}", hash, error)
            }
        }
        else {
            println!("No fileroot in config! Not rendering anything for {}!", &hash)
        }
    }
}}

pub fn customize(hbs: &mut Handlebars) {
    hbs.register_helper("imagelink", Box::new(imagelink_helper));
}
