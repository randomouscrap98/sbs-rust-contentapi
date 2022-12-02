use rocket_dyn_templates::Template;

use crate::context::*;
use crate::api_data::*;
use crate::api::*;
use crate::conversion;
use super::*;
use rocket::response::status::Custom as RocketCustom;

async fn forumroot_request (context: &Context) -> Result<RequestResult, ApiError>
{
    //The request which we will spend the entire function building
    let mut request = FullRequest::new();
    add_value!(request, "category_literal", SBSContentType::forumcategory.to_string());
    add_value!(request, "thread_literal", SBSContentType::forumthread.to_string());

    let mut category_request = minimal_content!(String::from("literalType = @category_literal"));
    category_request.name = Some(String::from("category"));
    request.requests.push(category_request);

    ////But what if we were passed preview?
    //if let Some(preview) = search.preview {
    //    let hashes: Vec<String> = preview.split(",").map(|h| String::from(h.trim())).collect();
    //    add_value!(request, "preview_hashes", hashes);
    //    let mut preview_request = minimal_content!(format!("{} and hash in @preview_hashes", base_query));
    //    preview_request.name = Some(String::from("preview"));
    //    request.requests.push(preview_request);
    //}

    //println!("Sending: {:?}", &request);

    post_request(context, &request).await
}

#[get("/forum")]
pub async fn forum_get(context: Context) -> Result<Template, RocketCustom<String>> {
    let result = forumroot_request(&context).await.map_err(rocket_error!())?;
    let categories = conversion::cast_result::<MinimalContent>(&result, "category").map_err(rocket_error!())?;
    Ok(basic_template!("forum", context, {}))
}
