use super::*;

use contentapi::Content;
use maud::DOCTYPE;
use serde::Serialize;

pub fn render(data: MainLayoutData, search: Search, images: Vec<Image>, previews: Vec<Image>, errors: Option<Vec<String>>) -> String {
    let sizes = vec![(1,"1"),(2,"2"),(3,"3")]; //This may seem stupid but I might want to change the text later
    let imagesize: i64 = 100 * search.size as i64; //Used to have + something, but now...

    layout(&data, html!{
        (DOCTYPE)
        html lang=(data.user_config.language) {
            head {
                (basic_meta(&data.config))
                title { "SmileBASIC Source Image Browser" }
                meta name="description" content="Simple image browser widget";
                (style(&data.config, "/base.css"))
                (script(&data.config, "/base.js"))
                (style(&data.config, "/forpage/imagebrowser.css"))
                (script(&data.config, "/forpage/imagebrowser.js"))
            }
            //This is meant to go in an iframe, so it will use up the whole space
            body data-size=(search.size){
                h3 { "Upload file:" }
                //This doesn't need an action since it's self posting but just in case...
                form method="POST" action="/widget/imagebrowser" enctype="multipart/form-data" {
                    (errorlist(errors))
                    input #"fileinput" type="file" name="file" class="largeinput" accept="image/*";
                    input type="submit" value="Upload";
                }
                hr;
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
                    @if search.preview.is_some() {
                        //Used to have h4 Preview images
                        div."imagelist" {
                            @if previews.len() > 0 {
                                (image_list(&data.config, previews, imagesize))
                            }
                            else {
                                p."aside" {"No images returned for preview!"}
                            }
                        }
                        hr;
                    }
                    //Used to have img navigation here
                    div."imagelist" {
                        @if images.len() > 0 {
                            (image_list(&data.config, images, imagesize))
                        }
                        else {
                            p."aside" {"No images!"}
                        }
                    }
                    (image_navigation(&data.config, search))
                }
            }
        }
    }).into_string()
}

#[derive(Serialize, Debug, Clone)]
pub struct Search
{
    #[serde(default = "one")]
    pub size: i32,
    pub global: bool,
    pub oldest: bool,
    #[serde(default)]
    pub page: i32,
    pub preview: Option<String>
}

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

fn one() -> i32 {1}

fn image_list(config: &LinkConfig, images: Vec<Image>, size: i64) -> Markup {
    html! {
        @for image in images {
            div."imagepreview" {
                a href=(base_image_link(config, &image.hash)) target="_blank" {
                    img src=(image_link(config, &image.hash, size as i64, false));
                }
                input."hover" readonly value=(image.hash) title=(image.hash);
            }
        }
    }
}

fn image_navigation(config: &LinkConfig, search: Search) -> Markup {
    let mut searchprev = search.clone();
    let mut searchnext = search.clone();
    searchprev.page = searchprev.page - 1;
    searchnext.page = searchnext.page + 1;
    html! {
        div."smallseparate browsepagenav" {
            @if let Ok(prevlink) = serde_urlencoded::to_string(searchprev) {
                a."coolbutton" href={(config.http_root)(prevlink)} {"Previous"}
            }
            @if let Ok(nextlink) = serde_urlencoded::to_string(searchnext) {
                a."coolbutton" href={(config.http_root)(nextlink)} {"Next"}
            }
        }
    }
}
