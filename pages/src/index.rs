use common::constants::SBSPageType;
use common::render::layout::*;
use common::response::*;
use common::*;
use maud::*;

//This will render the entire index! It's a handler WITH the template in it! Maybe that's kinda weird? who knows...
pub fn render(data: MainLayoutData, page_raw: Option<String>) -> String {
    layout(
        &data,
        html! {
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
        },
    )
    .into_string()
}

pub async fn get_render(context: PageContext) -> Result<Response, Error> {
    let mut pages = capi::get_systempage(&context, String::from(SBSPageType::FRONTPAGE))?;
    let mut frontpage: Option<String> = None;
    if let Some(page) = pages.pop() {
        frontpage = Some(page.text);
    }
    Ok(Response::Render(render(context.layout_data, frontpage)))
}

