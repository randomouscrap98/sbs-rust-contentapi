use rocket_dyn_templates::Template;
use crate::context::Context;
use super::*;


#[get("/")]
pub async fn index_get(context: Context) -> Result<Template, RouteError> {
    Ok(basic_template!("index", context, {}))
}

#[get("/about")] 
pub async fn about_get(context: Context) -> Result<Template, RouteError> {
    Ok(basic_template!("about", context, {}))
}
