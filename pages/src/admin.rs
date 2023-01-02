use common::*;
use common::render::*;
use common::render::layout::*;
use contentapi::forms::*;
//use contentapi::*;
//use contentapi::conversion::*;
use maud::*;
//use serde::{Serialize, Deserialize};

pub fn render(data: MainLayoutData, registration_config: RegistrationConfig, registrationconfig_errors: Option<Vec<String>>) -> String
{
    layout(&data, html!{
        section {
            @if let Some(user) = &data.user {
                @if user.admin {
                    h3 { "Banning:" }
                    p { "Go to the individual user's page to ban them" }
                    h3 { "Registration config:" }
                    form method="POST" action={(data.links.http_root)"/admin?registrationconfig"} {
                        (errorlist(registrationconfig_errors))
                        label."inline" for="registrationconfig_enabled"{
                            span{"Allow registration:"} 
                            input #"registrationconfig_enabled" type="checkbox" name="enabled" value="true" checked[registration_config.enabled];
                        }
                        input type="submit" value="Set (NO WARNING, BE CAREFUL!)";
                    }
                }
                @else {
                    p."error" { "You must be an admin to use this page!" }
                }
            }
            @else {
                p."error" { "You must be logged in to use this page!" }
            }
        }
    }).into_string()
}

pub async fn get_render(context: PageContext) -> Result<Response, Error> 
{
    let reg_config = context.api_context.get_registrationconfig().await?;
    Ok(Response::Render(render(context.layout_data, reg_config, None)))
}