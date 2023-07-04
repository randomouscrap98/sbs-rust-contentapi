use common::*;
use common::render::*;
use common::render::layout::*;
use common::response::*;

use contentapi::*;
use contentapi::forms::*;
use contentapi::endpoints::*;
use maud::*;
use serde::{Serialize, Deserialize};


pub fn render(data: MainLayoutData, search: Search, images: Vec<Image>, previews: Vec<Image>, errors: Option<Vec<String>>) -> String 
{
    let sizes = vec![(1,"1"),(2,"2"),(3,"3")]; //This may seem stupid but I might want to change the text later
    let imagesize: i64 = 100 * search.size as i64; //Used to have + something, but now...

    basic_skeleton(&data, html! {
        title { "SmileBASIC Source Image Browser" }
        meta name="description" content="Simple image browser widget";
        (data.links.style("/forpage/imagebrowser.css"))
        (data.links.script("/forpage/imagebrowser.js"))
    }, html! {
        //Might as well not show the upload form if user isn't logged in
        @if let Some(_user) = &data.user {
            h3 { "Upload file:" }
            //This doesn't need an action since it's self posting but just in case...
            form method="POST" action=(data.links.file_upload_root) enctype="multipart/form-data" {
                (errorlist(errors))
                @if let Some(error) = &search.error {
                    (errorlist(Some(vec![error.clone()])))
                }
                input #"fileinput" type="file" name="file" class="largeinput" accept="image/*";
                input type="hidden" name="token" value=[&data.user_token];
                input type="hidden" name="errorUrl" value={(data.current())"?error={{error}}"};
                input type="hidden" name="successUrl" value={(data.current())"?preview={{hash}}"};
                input type="submit" value="Upload";
            }
            hr;
        }
        h3{ "Browse files:" }
        div."scrollable" {
            //Don't include an action so it just posts to the same url but with the form as params...?
            form method="GET" id="browseform" {
                label."inline" for="search-preview" {
                    span { "Preview: " }
                    input."largeinput" #"search-preview" type="text" name="preview" value=[&search.preview] placeholder="Comma separated hashes";
                }
                label."inline" for="search-all" {
                    span { "Global search:" }
                    input #"search-all" type="checkbox" name="global" value="true" checked[search.global];
                }
                label."inline" for="search-oldest" {
                    span {"Oldest first:"}
                    input #"search-oldest" type="checkbox" name="oldest" value="true" checked[search.oldest];
                }
                label."inline" for="search-size" {
                    span{"Size:"}
                    select #"search-size" value=(search.size) name="size" {
                        @for (value,text) in sizes {
                            option value=(value) selected[value == search.size] { (text) }
                        }
                    }
                }
                label."inline" for="search-page" {
                    span {"Page:"}
                    input."smallinput" #"search-page" type="text" name="page" value=(search.page); 
                }
                input type="submit" value="Update search";
            }
            @if !is_empty(&search.preview) {
                //Used to have h4 Preview images
                div."imagelist" {
                    @if previews.len() > 0 {
                        (image_list(&data.links, previews, imagesize))
                    }
                    @else {
                        p."aside" {"No images returned for preview!"}
                    }
                }
                hr;
            }
            //Used to have img navigation here
            div."imagelist" {
                @if images.len() > 0 {
                    (image_list(&data.links, images, imagesize))
                }
                @else {
                    p."aside" {"No images!"}
                }
            }
            (image_navigation(&data, search))
        }
    }).into_string()
}

fn image_list(config: &LinkConfig, images: Vec<Image>, size: i64) -> Markup {
    html! {
        @for image in images {
            div."imagepreview" {
                a href=(config.image_default(&image.hash)) target="_blank" {
                    img src=(config.image(&image.hash, &QueryImage{ size: Some(size), crop: None }));
                }
                input."hover" readonly value=(image.hash) title=(image.hash);
            }
        }
    }
}

fn image_navigation(data: &MainLayoutData, search: Search) -> Markup {
    let mut searchprev = search.clone();
    let mut searchnext = search.clone();
    searchprev.page = searchprev.page - 1;
    searchnext.page = searchnext.page + 1;
    html! {
        div."smallseparate browsepagenav" {
            @if let Ok(prevlink) = serde_urlencoded::to_string(searchprev) {
                a."coolbutton" href={(data.current())"?"(prevlink)} {"Previous"}
            }
            @if let Ok(nextlink) = serde_urlencoded::to_string(searchnext) {
                a."coolbutton" href={(data.current())"?"(nextlink)} {"Next"}
            }
        }
    }
}



#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Search
{
    pub size: i32,
    pub global: bool,
    pub oldest: bool,
    pub page: i32,
    pub preview: Option<String>,
    pub error: Option<String>
}

impl Default for Search {
    fn default() -> Self {
        Self {
            size: 1,
            global: false,
            oldest: false,
            page: 0,
            preview: None,
            error: None
        }
    }
}


#[derive(Debug)]
pub struct Image 
{
    pub hash : String
}

impl From<Content> for Image {
    fn from(content: Content) -> Self {
        Self { 
            hash: if let Some(hash) = content.hash { hash } else { String::from("") }
        }
    }
}

async fn imagebrowser_request(context: &ApiContext, search: &Search, per_page: i32) -> Result<RequestResult, ApiError>
{
    //The request which we will spend the entire function building
    let mut request = FullRequest::new();
    add_value!(request, "type", ContentType::FILE);

    let base_query = "contentType = @type and !valuekeynotlike({{system}}) and !notdeleted()";
    let mut query = String::from(base_query);

    //Add user restriction to query
    if let Some(user) = context.get_me_safe().await {
        add_value!(request, "userId", user.id);
        if !search.global {
            query.push_str(" and createUserId = @userId");
        }
    }

    let fields = "id,hash,contentType,createUserId";
    let order = String::from(if search.oldest { "id" } else { "id_desc" });
    let main_request = build_request!(
        RequestType::content, 
        String::from(fields), 
        query, 
        order, 
        per_page,
        search.page * per_page
    ); 
    request.requests.push(main_request);

    //But what if we were passed preview?
    if let Some(ref preview) = search.preview {
        let hashes: Vec<String> = preview.split(",").map(|h| String::from(h.trim())).collect();
        add_value!(request, "preview_hashes", hashes);
        let mut preview_request = build_request!(
            RequestType::content, 
            String::from(fields),
            format!("{} and hash in @preview_hashes", base_query)
        );
        preview_request.name = Some(String::from("preview"));
        request.requests.push(preview_request);
    }

    //println!("Sending: {:?}", &request);

    context.post_request(&request).await
}


pub async fn query_render(context: PageContext, search: Search, per_page: i32) -> Result<Response,Error> {
    let result = imagebrowser_request(&context.api_context, &search, per_page).await?;
    let images = conversion::cast_result_safe::<Content>(&result, "content")?;
    let previews = conversion::cast_result_safe::<Content>(&result, "preview")?;

    Ok(Response::Render(render(context.layout_data, search, 
        images.into_iter().map(|i| i.into()).collect(), 
        previews.into_iter().map(|i| i.into()).collect(), None)))
}