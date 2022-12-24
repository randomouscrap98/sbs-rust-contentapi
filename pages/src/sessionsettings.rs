
use common::render::*;
use common::*;
use common::render::layout::*;
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
                label."inline" for="settings-theme" 
                {
                    span{"Theme:"}
                    select #"settings-theme" name="theme" {
                        @for (key,value) in constants::USERTHEMES {
                            option value=(key) selected[&data.user_config.theme == key] { (value) }
                        }
                    }
                }
                label."inline" for="settings-compact" {
                    span { "Compact mode: " }
                    input."" #"settings-compact" type="checkbox" name="compact" checked[settings.compact] value="true";
                }
                input type="submit" value="Save";
            }
            p."aside" { "These settings are persisted in a cookie and only available on this device" }
        }
    }).into_string()
}

//pub fn render(data: MainLayoutData, form: UserConfig) -> Result<String>
//{
//
//}