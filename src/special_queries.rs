use crate::forms::ImageBrowseSearch; 
use crate::api_data::*;
use crate::context::Context;
use crate::api::*;


pub async fn imagebrowser_request(context: &Context, search: &ImageBrowseSearch<'_>) -> Result<RequestResult, ApiError>
{
    //The request which we will spend the entire function building
    let mut request = FullRequest::new();
    add_value!(request, "type", ContentType::FILE);

    let base_query = "contentType = @type";
    let mut query = String::from(base_query);

    //Add user restriction to query
    if let Some(user) = get_user_safe(context).await {
        add_value!(request, "userId", user.id);
        if !search.global {
            query.push_str(" and createUserId = @userId");
        }
    }

    let mut main_request = minimal_content!(query);
    main_request.limit = context.config.default_imagebrowser_count.into();

    //Invert order if we want oldest first
    if search.oldest {
        main_request.order = Some(String::from("id_desc"));
    }

    request.requests.push(main_request);

    //But what if we were passed preview?
    if let Some(preview) = search.preview {
        let hashes: Vec<String> = preview.split(",").map(|h| String::from(h.trim())).collect();
        add_value!(request, "preview_hashes", hashes);
        let mut preview_request = minimal_content!(format!("{} and hash in @preview_hashes", base_query));
        preview_request.name = Some(String::from("preview"));
        request.requests.push(preview_request);
    }

    println!("Sending: {:?}", &request);

    post_request(context, &request).await
}
