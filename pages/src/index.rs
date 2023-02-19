use common::*;
use common::render::layout::*;
use maud::*;

//This will render the entire index! It's a handler WITH the template in it! Maybe that's kinda weird? who knows...
//pub fn index(data: MainLayoutData) -> Result<impl warp::Reply, Infallible>{
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
            /*h1 { "Hello, this is the index!" }
            p { "This is a demo of the new SBS frontend! Changes you make here will NOT be saved "
                "when the final move is made! However, most user-related features will work, so you "     
                "can test recovering your account (all passwords are reset) and browsing the forums. "
                "Eventually, there'll be more, just remember that " b{"NOTHING"} " done here will be "
                "reflected on " a href="https://old.smilebasicsource.com"{"old.smilebasicsource.com"} ", "
                "and even if you recover your account now, you will still need to do it at the final "
                "transition."
            }*/
        }
    }).into_string()
}

pub async fn get_render(mut context: PageContext) -> Result<Response, Error> {
    let frontpage = prefab::get_system_frontpage(&mut context.api_context).await?;
    Ok(Response::Render(render(context.layout_data, frontpage.and_then(|x| x.text))))
}