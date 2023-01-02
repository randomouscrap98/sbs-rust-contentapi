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
                    form method="POST" action={(data.links.http_root)"/admin?registrationconfig=1"} {
                        (errorlist(registrationconfig_errors))
                        label."inline" for="registrationconfig_enabled"{
                            span{"Allow registration:"} 
                            input #"registrationconfig_enabled" type="checkbox" name="enabled" value="true" checked[registration_config.enabled];
                        }
                        p."error aside" { 
                            "WARN: these settings are temporary and are reset when the server is reset! To make permanent "
                            "changes, talk to the SBS system admin!"
                        }
                        input type="submit" value="Set (NO WARNING, BE CAREFUL!)";
                    }
                    h3 { "Admin log:" }
                    p { "Eventually!" }
                    h3 { "Active bans:" }
                    p { "Eventually!" }
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

async fn get_render_internal(context: PageContext, registrationconfig_errors: Option<Vec<String>>) -> 
    Result<Response, Error>
{
    let reg_config = context.api_context.get_registrationconfig().await?;
    Ok(Response::Render(render(context.layout_data, reg_config, registrationconfig_errors)))
}

pub async fn get_render(context: PageContext) -> Result<Response, Error> 
{
    get_render_internal(context, None).await
}

pub async fn post_registrationconfig(context: PageContext, form: RegistrationConfig) -> Result<Response, Error>
{
    let mut errors = Vec::new();
    match context.api_context.post_registrationconfig(&form).await {
        Ok(_token) => {} //Don't need the token
        Err(error) => { errors.push(error.to_user_string()) }
    };
    get_render_internal(context, Some(errors)).await
}