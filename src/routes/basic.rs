use rocket_dyn_templates::Template;
use crate::context::Context;
use super::*;

use rocket::response::status::Custom as RocketCustom;


#[get("/")]
pub async fn index_get(context: Context) -> Result<Template, RocketCustom<String>> {
    Ok(basic_template!("index", context, {}))
}

#[get("/about")] 
pub async fn about_get(context: Context) -> Result<Template, RocketCustom<String>> {
    Ok(basic_template!("about", context, {}))
}
