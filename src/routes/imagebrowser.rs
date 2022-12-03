use crate::context::*;
use crate::forms;
use crate::api_data::*;
use crate::api::*;
use crate::conversion;
use crate::hbs_custom;
use super::*;
use rocket::form::Form;
use rocket_dyn_templates::Template;


async fn imagebrowser_request(context: &Context, search: &forms::ImageBrowseSearch<'_>) -> Result<RequestResult, ApiError>
{
    //The request which we will spend the entire function building
    let mut request = FullRequest::new();
    add_value!(request, "type", ContentType::FILE);

    let base_query = "contentType = @type and !valuekeynotlike({{system}}) and !notdeleted()";
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
    main_request.skip = (search.page as i64) * main_request.limit;

    //Oldest means default ordering, so 'not' oldest is actually inverted order
    if !search.oldest {
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

    //println!("Sending: {:?}", &request);

    post_request(context, &request).await
}

async fn widget_imagebrowser_base(context: &Context, search: &forms::ImageBrowseSearch<'_>, errors: Option<Vec::<String>>) -> Result<Template, RouteError>
{
    let result = imagebrowser_request(context, search).await?;
    let images = conversion::cast_result_safe::<MinimalContent>(&result, "content")?;
    let previews = conversion::cast_result_safe::<MinimalContent>(&result, "preview")?;
    let mut searchprev = search.clone();
    let mut searchnext = search.clone();
    searchprev.page = searchprev.page - 1;
    searchnext.page = searchnext.page + 1;

    Ok(basic_template!("widgets/imagebrowser", context, {
        search : &search,
        haspreview : previews.len() > 0,
        hasimages : images.len() > 0,
        previewimages : previews,
        imagesize: 0 + 100 * search.size,
        nextpagelink : if let Ok(q) = serde_qs::to_string(&searchnext) { 
            Some(format!("{}?{}", context.route_path, q)) } else { None },
        previouspagelink : if searchprev.page >= 0 { if let Ok(q) = serde_qs::to_string(&searchprev) {
            Some(format!("{}?{}", context.route_path, q)) } else { None } } else { None },
        images : images,
        errors : errors,
        sizevalues : vec![
            hbs_custom::SelectValue::new(1, "1", search.size), 
            hbs_custom::SelectValue::new(2, "2", search.size),
            hbs_custom::SelectValue::new(3, "3", search.size)
        ]
    }))
}

#[get("/widget/imagebrowser?<search..>")]
pub async fn widget_imagebrowser_get(context: Context, search: forms::ImageBrowseSearch<'_>) -> Result<Template, RouteError> 
{
    widget_imagebrowser_base(&context, &search, None).await
}

#[post("/widget/imagebrowser", data = "<upload>")]
pub async fn widget_imagebrowser_post(context: Context, mut upload: Form<forms::FileUpload<'_>>) -> Result<Template, RouteError> 
{
    let mut search = forms::ImageBrowseSearch::new();
    match upload_file(&context, &mut upload).await
    {
        Ok(result) => {
            search.preview = Some(&result.hash); 
            widget_imagebrowser_base(&context, &search, None).await
        }
        Err(error) => { 
            widget_imagebrowser_base(&context, &search, Some(vec![error.to_string()])).await
        }
    }
}