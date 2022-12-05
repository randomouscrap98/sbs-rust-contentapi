use crate::context::*;
use crate::forms;
use crate::api_data::*;
use crate::api::*;
use crate::conversion;
use crate::hbs_custom;
use super::*;
use rocket::form::Form;
use rocket_dyn_templates::Template;

#[get("/widget/bbcodepreview")]
pub async fn widget_bbcodepreview_get(context: Context) -> Result<Template, RouteError> 
{
}

#[post("/widget/bbcodepreview", data = "<result>")]
pub async fn widget_bbcodepreview_post(context: Context, result: Form<forms::FileUpload<'_>>) -> Result<Template, RouteError> 
{
    let mut search = forms::ImageBrowseSearch::new();
    match upload_file(&context, &mut upload).await
    {
        Ok(result) => {
            search.preview = result.hash;
            widget_imagebrowser_base(&context, &search, None).await
        }
        Err(error) => { 
            widget_imagebrowser_base(&context, &search, Some(vec![error.to_string()])).await
        }
    }
}