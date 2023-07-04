
use common::*;
use common::constants::SBSPageType;
use common::prefab::{get_all_documentation, get_system_docscustom, DOCPARENTMINIMALFIELDS, get_documentation_parent};
use common::render::forum::display_doctree;
use common::render::layout::*;
use common::response::*;
use contentapi::*;
use contentapi::permissions::can_user_action;
use maud::*;

//This will render the entire index! It's a handler WITH the template in it! Maybe that's kinda weird? who knows...
//pub fn index(data: MainLayoutData) -> Result<impl warp::Reply, Infallible>{
pub fn render(data: MainLayoutData, documentation: &Vec<Content>, docparent: Content, docscustom: Option<Content>) -> String {
    layout(&data, html!{
        (data.links.style("/forpage/forum.css"))
        (data.links.script("/forpage/forum.js"))
        section {
            @if let Some(docscustom) = docscustom {
                (PreEscaped(opt_s!(docscustom.text)))
            }
            (display_doctree(&data, documentation, 1))
            @if let Some(ref user) = data.user {
                @if can_user_action(user, "C", &docparent) {
                    div."pagelist smallseparate" {
                        a."coolbutton" #"newdocumentation" href=(data.links.page_editor_new(SBSPageType::DOCUMENTATION)) { "New Documentation" }
                    }
                }
            }
        }
    }).into_string()
}

pub async fn get_render(mut context: PageContext) -> Result<Response, Error> {
    let documentation = get_all_documentation(&mut context.api_context).await?;
    let docparent = get_documentation_parent(&mut context.api_context, DOCPARENTMINIMALFIELDS).await?;
    let docscustom = get_system_docscustom(&mut context.api_context).await?;
    Ok(Response::Render(render(context.layout_data, &documentation, docparent, docscustom)))
}