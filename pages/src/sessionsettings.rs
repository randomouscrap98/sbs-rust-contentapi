
use common::*;
use common::layout::*;
use maud::*;

pub fn render(data: MainLayoutData) -> String 
{
    let settings = &data.user_config;
    //Need to split category search into parts 
    //let search_system = match &search.system { Some(system) => system, None => };
    layout(&data, html!{
        section {
            h1 { "Local session settings" }
            p."aside" { "These settings are persisted in a cookie and only available on this device" }
            form method="POST" action={(data.config.http_root)"/sessionsettings"} {
                label."inline" for="settings-compact" {
                    span { "Compact mode: " }
                    input."" #"settings-compact" type="checkbox" name="compact" checked[settings.compact] value="true";
                }
                input type="submit" value="Save";
            }
        }
    }).into_string()
}

//pub fn render(data: MainLayoutData, form: UserConfig) -> Result<String>
//{
//
//}