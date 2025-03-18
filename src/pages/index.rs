use maud::*;

use super::common::contentapi::*;
use super::context::*;
use crate::layout::*;
use crate::response::*;

static SYSTEMTYPE: i8 = 5;
//(FRONTPAGE:"frontpage"),

//This will render the entire index! It's a handler WITH the template in it! Maybe that's kinda weird? who knows...
fn render(data: MainLayoutData, page_raw: Option<String>) -> String {
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

// Get any system content with given literaltype
fn get_systempage(ctx: &PageContext, literal_type: String) -> Result<Vec<BasicContent>, Error> {
    let query = format!(
        "SELECT {} FROM content c {} WHERE c.contentType = ? AND c.literalType = ?",
        BASICCONTENTFIELDS,
        join_common_content("c.id")
    );
    let mut stmt = ctx.dbcon.prepare(&query)?;
    gather_basiccontent(
        (&mut stmt, &query),
        rusqlite::params![SYSTEMTYPE, &literal_type],
    )
}

pub async fn get_render(context: PageContext) -> Result<Response, Error> {
    let mut pages = get_systempage(&context, String::from(SBSPageType::FRONTPAGE))?;
    let mut frontpage: Option<String> = None;
    if let Some(page) = pages.pop() {
        frontpage = Some(page.text);
    }
    Ok(Response::Render(render(context.layout_data, frontpage)))
}
