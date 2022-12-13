
use bbcode::BBCode;
use contentapi::{User, Content, add_value, build_request, FullRequest, RequestType};

use super::*;

pub fn render(data: MainLayoutData, mut bbcode: BBCode, user: Option<User>, userpage: Option<Content>) -> String {
    layout(&data, html!{
        (style(&data.config, "/forpage/user.css"))
        section {
            @if let Some(user) = user {
                div #"pageuser" {
                    img src={(image_link(&data.config, &user.avatar, 300, true))};
                    div #"infoblock" {
                        h1 {(user.username)}
                        div."aside mediumseparate" #"userinfo" {
                            // Some info about the user 
                            div { "Member since: " time { (user.createDate.to_rfc3339()) } }
                            div { "ID: "(user.id) }
                        }
                        //If the user has no bio, that's ok! 
                        @if let Some(userpage) = userpage {
                            div."content" #"userbio" { (PreEscaped(bbcode.parse_maybeprofiled(s(&userpage.text), format!("userpage-{}", i(&userpage.id))))) } 
                        }
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


pub async fn get_render(context: PageContext, username: String) -> Result<Response, Error>
{
    //Go get the user and their userpage
    let mut request = FullRequest::new();
    add_value!(request, "username", username);

    request.requests.push(build_request!(
        RequestType::user, 
        String::from("*"), 
        String::from("username = @username")
    )); 

    request.requests.push(build_request!(
        RequestType::content, 
        String::from("*"), //ok do we really need it ALL?
        String::from("!userpage(@user.id)")
    )); 

    let result = context.api_context.post_request(&request).await?;

    //Now try to parse two things out of it
    let mut users_raw = contentapi::conversion::cast_result_required::<User>(&result, "user")?;
    let mut content_raw = contentapi::conversion::cast_result_required::<Content>(&result, "content")?;

    Ok(Response::Render(render(context.layout_data, context.bbcode, users_raw.pop(), content_raw.pop())))
}
