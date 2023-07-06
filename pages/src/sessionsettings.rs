
use common::render::*;
use common::*;
use common::render::layout::*;
use common::response::*;
use maud::*;

pub fn render(data: MainLayoutData, errors: Option<Vec<String>>) -> String 
{
    let settings = &data.user_config;
    //Need to split category search into parts 
    //let search_system = match &search.system { Some(system) => system, None => };
    layout(&data, html!{
        section {
            h1 { "Local session settings" }
            form method="POST" action={(data.links.http_root)"/sessionsettings"} {
                (errorlist(errors))
                div."inline smallseparate" {
                    label for="settings-theme" {"Theme:"}
                    select #"settings-theme" name="theme" {
                        @for (key,value) in constants::USERTHEMES {
                            option value=(key) selected[&data.user_config.theme == key] { (value) }
                        }
                    }
                }
                div."inline smallseparate" {
                    label for="settings-compact" { "Compact mode: " }
                    input."" #"settings-compact" type="checkbox" name="compact" checked[settings.compact] value="true";
                }
                div."inline smallseparate" {
                    label for="settings-toppaginationposts" { "Top Pagination (posts): " }
                    input."" #"settings-toppaginationposts" type="checkbox" name="toppagination_posts" checked[settings.toppagination_posts] value="true";
                }
                input type="submit" value="Save";
            }
            p."aside" { "These settings are persisted in a cookie and only available on this device" }
        }
    }).into_string()
}

pub async fn get_render(context: PageContext) -> Result<Response, Error> {
    Ok(Response::Render(render(context.layout_data, None)))
}