
use contentapi::{User, Content};

use super::*;

pub fn render(data: MainLayoutData, user: Option<User>, userpage: Option<Content>) -> String {
    layout(&data, html!{
        (style(&data.config, "/forpage/user.css"))
        section {
            @if let Some(user) = user {
                div #"pageuser" {
                    img src={(image_link(&data.config, user.avatar, 300, true))};
                    div #"infoblock" {
                        h1 {(user.username)}
                        div."aside mediumseparate" #"userinfo" {
                            // Some info about the user 
                            div { "Member since: " time { (user.createDate) } }
                            div { "ID: "(user.id) }
                        }
                        //If the user has no bio, that's ok! 
                        div."content" #"userbio" { PreEscaped(bbcode(&userpage.text)) } 
                    }
                }
            }
            @else {
                //Maybe we do this OR a 404? IDK which one?
                p."error" {"Couldn't find that user!"}
            }
        }
    }).into_string()
}
