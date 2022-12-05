use crate::context::*;
use crate::forms;
use crate::bbcode::*;
use super::*;
use rocket::State;
use rocket::form::Form;
use rocket_dyn_templates::Template;

#[get("/widget/bbcodepreview")]
pub async fn widget_bbcodepreview_get(context: Context) -> Result<Template, RouteError> 
{
    Ok(basic_template!("widgets/bbcodepreview", context, {}))
}

#[post("/widget/bbcodepreview", data = "<test>")]
pub async fn widget_bbcodepreview_post(context: Context, test: Form<forms::BasicText<'_>>, bbcode: &State<BBCode>) -> Result<Template, RouteError> 
{
    Ok(basic_template!("widgets/bbcodepreview", context, {
        result : bbcode.parse(test.text)
    }))
}