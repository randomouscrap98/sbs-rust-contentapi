
use super::*;
use contentapi::*;
use contentapi::endpoints::*;

//use serde::{Serialize, Deserialize};
//use serde_aux::prelude::deserialize_bool_from_anything;

pub fn render(data: MainLayoutData, private: Option<contentapi::UserPrivate>, userbio: Option<Content>,
    update_errors: Option<Vec<String>>, bio_errors: Option<Vec<String>>, private_errors: Option<Vec<String>>) -> String 
{
    let mut bio_id: i64 = 0;
    let mut bio_text: String = String::from("");
    let mut email: String = String::from("");
    if let Some(bio) = userbio {
        if let Some(id) = bio.id { bio_id = id }
        if let Some(text) = bio.text { bio_text = text.clone() }
    }
    if let Some(private) = private {
        email = private.email.clone();
    }
    layout(&data, html!{
        @if let Some(user) = &data.user {
            (style(&data.config, "/forpage/userhome.css"))
            section {
                h1 {(user.username)}
                div #"userhomeinfo" {
                    img src=(image_link(&data.config, &user.avatar, 300, true));
                    div #"infoblock" {
                        table data-special=(s(&user.special)) data-type=(user.r#type) {
                            tr { td { b { "Email:"} } td."spoilertext"{(email)} } 
                            tr { td { b { "User ID:"} } td {(user.id)} }
                            tr { td { b { "Joined:"} } td { time {(user.createDate.to_string())} } }
                            tr { td { b { "Avatar:"} } td {(user.avatar)} }
                            tr { td { b { "Admin:"} } td {(b(user.admin))} }//{{{#if user.admin}}true{{else}}false{{/if}}</td></tr>
                        }
                        //Might turn this into a collbutton
                        div."smallseparate" #"userlinks" {
                            a."flatlink" #"publiclink" href={(data.config.http_root)"/user/"(user.username)} {"User page"}
                            span{"/"}
                            a."flatlink" #"logoutlink" href={(data.config.http_root)"/logout"} {"Logout"}
                        }
                    }
                }
                hr;
                h3 #"update-userbio" {"Update bio:"}
                form method="POST" action={(data.current_path)"?bio#update-userbio"} {
                    (errorlist(bio_errors))
                    input type="hidden" name="id" value=(bio_id);
                    textarea #"update_userbio" type="text" name="text"{(bio_text)}
                    input type="submit" value="Update";
                }
                hr;
                h3 #"update-user"{"Update info:"}
                form method="POST" action={(data.current_path)"#update-user"} { 
                    (errorlist(update_errors))
                    label for="update_username"{"Username:"}
                    input #"update_username" type="text" name="username" value=(user.username);
                    label for="update_avatar"{"Avatar:"}
                    input #"update_avatar" type="text" name="avatar" value=(user.avatar);
                    p."aside"{"Copy key/hash from image browser below"}
                    input type="submit" value="Update";
                }
            }
            section {
                iframe."imagebrowser" src={(data.config.http_root)"/widget/imagebrowser"} {}
            }
            section {
                h3 #"update-sensitive"{"Update sensitive info"}
                p{"Only set the fields you want to change, except 'current password', which is required"}
                form method="POST" action={(data.current_path)"?sensitive#update-sensitive"} autocomplete="off" {
                    (errorlist(private_errors))
                    //<label for="sensitive_username">New Username:</label>
                    //<input id="sensitve_username" type="text" autocomplete="new-password" name="username" value="">
                    label for="sensitive_password"{"New Password:"}
                    input #"sensitive_password" type="password" autocomplete="new-password" name="password" value="";
                    label for="sensitive_email"{"New Email:"}
                    input #"sensitive_email" type="email" autocomplete="new-password" name="email" value="";
                    label for="sensitive_currentpassword"{ b{"Current Password:"} span."error"{"*"} }
                    input #"sensitive_currentpassword" type="password" required="" name="currentPassword";
                    input type="hidden" required="" name="currentEmail" value=(email);
                    input type="submit" value="Update";
                }
            }
        }
        @else {
            section {
                p."error"{"You must be logged in to see userhome!"}
            }
        }
    }).into_string()
}


pub async fn get_render(data: MainLayoutData, context: &ApiContext) -> Result<Response,Error> 
{
    let private = context.get_user_private_safe().await;
    let mut userpage : Option<Content> = None;

    if let Some(ref user) = data.user {
        let mut request = FullRequest::new();
        add_value!(request, "uid", user.id);
        let mut user_request = build_request!(
            RequestType::content, 
            String::from("*"), //ok do we really need it ALL?
            String::from("!userpage(@uid)")
        ); 
        user_request.name = Some(String::from("userpage"));
        request.requests.push(user_request);

        let result = context.post_request(&request).await?;

        let mut userpage_raw = conversion::cast_result_safe::<Content>(&result, "userpage")?;
        userpage = userpage_raw.pop(); //Doesn't matter if it's none
    }

    //pub fn render(data: MainLayoutData, private: contentapi::UserPrivate, userbio: Option<Content>,
    //update_errors: Option<Vec<String>>, bio_errors: Option<Vec<String>>, private_errors: Option<Vec<String>>) -> String 

    Ok(Response::Render(render(data, private, userpage, None, None, None)))
}