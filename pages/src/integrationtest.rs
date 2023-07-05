use common::*;
use common::render::layout::*;
use common::response::*;
use maud::html;

pub fn render(data: MainLayoutData) -> String 
{
    let mut errors : Vec<&'static str> = Vec::new();

    //Calculate errors before rendering page to make life easier
    if !data.links.http_root.starts_with("http://localhost") {
        errors.push("Frontend isn't running locally with non-empty http_root"); 
    }
    if data.user.is_some() { 
        errors.push("You must be logged out"); 
    }

    layout(&data, html!{
        style { r#"
            #testframe {
                width: 100%;
                height: 60vh;
            }  
        "# }
        (data.links.script("/forpage/integrationtest.js"))
        section {
            h1 { "TESTING PAGE" }
            p {
                "To run this testing page, the following must be met:"
                ul {
                    li { "Running frontend locally (localhost)" }
                    li { "Running backend locally" }
                    li { "Not logged in (try private window?) " a href={(data.links.http_root)"/logout"} {"Logout"}}
                    li { "Registration set to standard (it's NOT in the default settings!)" }
                    li { "ALL rate limiting turned off!" }
                    li { "Email handler set to file or null" }
                    li { "Backdoor registration 'get code endpoint' active" }
                    li { "Backdoor super updates allowed (this is A VERY dangerous setting!)" }
                    li { "Pre-existing data available (at least structure, categories, etc. not a fresh database)" }
                }
            }
            @if errors.is_empty() {
                a href="#" #"teststart" /*onclick="runtests();"*/ { "Loading tests..." }
                p."aside" { "Tests can take a very long time" }
                iframe #"testframe" /*onload="testonload();"*/ { }
            }
            @else {
                h3 { "This testing page isn't available because:" }
                ul."errors" {
                    @for error in errors {
                        li."error" { (error) }
                    }
                }
            }
        }
    }).into_string()
}

pub async fn get_render(context: PageContext) -> Result<Response, Error> {
    Ok(Response::Render(render(context.layout_data)))
}