use common::prefab::get_all_documentation;
use common::render::forum::display_doctree;
use common::render::layout::*;
use common::response::*;
use common::*;
use contentapi::*;
use maud::*;

//This will render the entire index! It's a handler WITH the template in it! Maybe that's kinda weird? who knows...
//pub fn index(data: MainLayoutData) -> Result<impl warp::Reply, Infallible>{
pub fn render(data: MainLayoutData, documentation: &Vec<Content>) -> String {
    layout(
        &data,
        html! {
            (data.links.style("/forpage/forum.css"))
            (data.links.script("/forpage/forum.js"))
            section {
                (display_doctree(&data, documentation, 1))
            }
        },
    )
    .into_string()
}

pub async fn get_render(mut context: PageContext) -> Result<Response, Error> {
    let documentation = get_all_documentation(&mut context.api_context).await?;
    Ok(Response::Render(render(
        context.layout_data,
        &documentation,
    )))
}
