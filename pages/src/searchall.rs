use common::*;
use common::constants::SBSPageType;
use common::render::forum::display_doctree;
use common::render::layout::*;
use contentapi::*;
use contentapi::permissions::can_user_action;
use maud::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct SearchAllForm {
    pub search : Option<String>
}

enum SearchAllResult {
    User(contentapi::User),
    Content(contentapi::Content)
}

//This will render the entire index! It's a handler WITH the template in it! Maybe that's kinda weird? who knows...
//pub fn index(data: MainLayoutData) -> Result<impl warp::Reply, Infallible>{
pub fn render(data: MainLayoutData, search_results: Vec<SearchAllResult>, search_form: SearchAllForm) -> String {
    layout(&data, html!{
        (data.links.style("/forpage/forum.css"))
        (data.links.style("/forpage/searchall.css"))
        section {
            //Want this to be enter to submit, don't remember if that's the default
            form #"searchallform" method="GET" {
                h3 { "Search website content" }
                div #"searchline" {
                    input #"searchinput" type="text" value=(opt_s!(search_form.search)) placeholder="Search";
                    input type="submit" value="🔎";
                }
            }
            div #"searchresults" {

            }
        }
    }).into_string()
}

//There is no post, searching is done in the GET params

pub async fn get_render(mut context: PageContext, search_form: SearchAllForm) -> Result<Response, Error> 
{

    Ok(Response::Render(render(context.layout_data, , search_form)))
}
