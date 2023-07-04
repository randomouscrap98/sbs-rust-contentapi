use common::*;
use common::render::layout::*;
use common::response::*;
use maud::*;

//This will render the entire index! It's a handler WITH the template in it! Maybe that's kinda weird? who knows...
pub fn render(data: MainLayoutData, page_raw: Option<String>) -> String {
    layout(&data, html!{
        //This is the body of index
        section {
            @if let Some(page) = page_raw {
                (PreEscaped(page))
            }
            @else {
                h1 { "Welcome to SmileBASIC Source!" }
                p."aside" { "Looks like the admins didn't set a frontpage..."}
            }
        }
    }).into_string()
}

pub async fn get_render(mut context: PageContext) -> Result<Response, Error> {
    let frontpage = prefab::get_system_frontpage(&mut context.api_context).await?;
    Ok(Response::Render(render(context.layout_data, frontpage.and_then(|x| x.text))))
}