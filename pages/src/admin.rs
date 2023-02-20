use common::*;
use common::constants::SBSPageType;
use common::forms::BasicPage;
use common::render::*;
use common::prefab::*;
use common::render::layout::*;
use contentapi::forms::*;
use contentapi::*;

//use contentapi::*;
//use contentapi::conversion::*;
use maud::*;
//use serde::{Serialize, Deserialize};

pub fn render(data: MainLayoutData, frontpage: Option<Content>, banner: Option<Content>, registration_config: RegistrationConfig, 
    registrationconfig_errors: Option<Vec<String>>, frontpage_errors: Option<Vec<String>>, banner_errors: Option<Vec<String>>) -> String
{
    let mut frontpage_id: i64 = 0;
    let mut frontpage_text: String = String::from("");
    if let Some(frontpage) = frontpage {
        if let Some(id) = frontpage.id { frontpage_id = id }
        if let Some(text) = frontpage.text { frontpage_text = text.clone() }
    }
    let mut banner_id: i64 = 0;
    let mut banner_text: String = String::from("");
    if let Some(banner) = banner {
        if let Some(id) = banner.id { banner_id = id }
        if let Some(text) = banner.text { banner_text = text.clone() }
    }
    layout(&data, html!{
        section {
            @if let Some(user) = &data.user {
                @if user.admin {
                    h3 { "Banning:" }
                    p { "Go to the individual user's page to ban them" }
                    hr;
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
                    hr;
                    h3 { "Admin log:" }
                    p { "Eventually!" }
                    hr;
                    h3 { "Active bans:" }
                    p { "Eventually!" }
                    hr;
                    h3 #"update-frontpage" {"Set frontpage (HTML!):"}
                    form."editor" method="POST" action={(data.links.http_root)"/admin?frontpage=1#update-frontpage"} {
                        (errorlist(frontpage_errors))
                        input type="hidden" name="id" value=(frontpage_id);
                        textarea type="text" name="text"{(frontpage_text)}
                        input type="submit" value="Update";
                    }
                    h3 #"update-alert" {"Set alert banner (HTML!):"}
                    form."editor" method="POST" action={(data.links.http_root)"/admin?alert=1#update-alert"} {
                        (errorlist(banner_errors))
                        input type="hidden" name="id" value=(banner_id);
                        textarea type="text" name="text"{(banner_text)}
                        input type="submit" value="Update";
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

async fn get_render_internal(mut context: PageContext, registrationconfig_errors: Option<Vec<String>>,
    frontpage_errors: Option<Vec<String>>, banner_errors: Option<Vec<String>>) -> Result<Response, Error>
{
    let reg_config = context.api_context.get_registrationconfig().await?;
    let frontpage = get_system_frontpage(&mut context.api_context).await?;
    let banner = get_system_alert(&mut context.api_context).await?;
    Ok(Response::Render(render(context.layout_data, frontpage, banner, reg_config, 
        registrationconfig_errors, frontpage_errors, banner_errors)))
}

pub async fn get_render(context: PageContext) -> Result<Response, Error> 
{
    get_render_internal(context, None, None, None).await
}

pub async fn post_registrationconfig(context: PageContext, form: RegistrationConfig) -> Result<Response, Error>
{
    let mut errors = Vec::new();
    match context.api_context.post_registrationconfig(&form).await {
        Ok(_token) => {} //Don't need the token
        Err(error) => { errors.push(error.to_user_string()) }
    };
    get_render_internal(context, Some(errors), None, None).await
}

async fn to_system_content(form: BasicPage, name: String, literal_type: String) -> Result<Content, Error> {
    let mut content = Content::default();
    //note: the hash it autogenerated from the name (hopefully)
    content.text = Some(form.text);
    content.id = Some(form.id);
    content.contentType = Some(ContentType::SYSTEM);
    content.name = Some(name);
    content.literalType = Some(literal_type);
    content.permissions = Some(make_permissions! { "0": "R" });
    content.values = Some(make_values! { "markup": "html" });
    Ok(content)
}

pub async fn post_frontpage(context: PageContext, form: BasicPage) -> Result<Response, Error>
{
    let mut errors = Vec::new();

    let content = to_system_content(form, String::from("frontpage"), SBSPageType::FRONTPAGE.to_string()).await?;

    match context.api_context.post_content(&content).await {
        Ok(_) => {} 
        Err(error) => { errors.push(error.to_user_string()) }
    };
    get_render_internal(context, None, Some(errors), None).await
}

pub async fn post_alert(mut context: PageContext, form: BasicPage) -> Result<Response, Error>
{
    let mut errors = Vec::new();

    let content = to_system_content(form, String::from("alert"), SBSPageType::ALERT.to_string()).await?;

    match context.api_context.post_content(&content).await {
        Ok(new_alert) => { context.layout_data.raw_alert = new_alert.text; } 
        Err(error) => { errors.push(error.to_user_string()) }
    };
    get_render_internal(context, None, None, Some(errors)).await
}
