use common::*;
use common::forms::BasicPage;
use common::forms::UserUpdate;
use common::render::*;
use common::render::layout::*;
use common::response::*;
use contentapi::*;
use contentapi::endpoints::*;
use contentapi::forms::*;
use maud::*;


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
            (data.links.style("/forpage/userhome.css"))
            section {
                h1 {(user.username)}
                div #"userhomeinfo" {
                    img src=(data.links.image(&user.avatar, &QueryImage::avatar(300)));
                    div #"infoblock" {
                        table data-special=(opt_s!(&user.special)) data-type=(user.r#type) {
                            tr { td { b { "Email:"} } td."spoilertext"{(email)} } 
                            tr { td { b { "User ID:"} } td {(user.id)} }
                            tr { td { b { "Joined:"} } td { time {(user.createDate.to_rfc3339())} } }
                            tr { td { b { "Avatar:"} } td {(user.avatar)} }
                            tr { td { b { "Admin:"} } td {(b(user.admin))} }//{{{#if user.admin}}true{{else}}false{{/if}}</td></tr>
                        }
                        //Might turn this into a collbutton
                        div."smallseparate" #"userlinks" {
                            a."flatlink" #"publiclink" href={(data.links.http_root)"/user/"(user.username)} {"User page"}
                            span{"/"}
                            a."flatlink" #"privatethreadslink" href=(data.links.forum_category_unsafe("private-threads")) {"Private Threads"}
                            span{"/"}
                            a."flatlink" #"logoutlink" href={(data.links.http_root)"/logout"} {"Logout"}
                        }
                    }
                }
                hr;
                h3 #"update-userbio" {"Update bio:"}
                // "Editor" forms are special forms which are meant for editing content instead of whatever other 
                //  forms do.
                form."editor" method="POST" action={(data.links.http_root)"/userhome?bio=1#update-userbio"} {
                    (errorlist(bio_errors))
                    input type="hidden" name="id" value=(bio_id);
                    textarea #"update_userbio" type="text" name="text"{(bio_text)}
                    input type="submit" value="Update";
                }
                hr;
                h3 #"update-user"{"Update info:"}
                form method="POST" action={(data.links.http_root)"/userhome#update-user"} { 
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
                iframe."imagebrowser" src={(data.links.imagebrowser())} {}
            }
            section {
                h3 #"update-sensitive"{"Update sensitive info"}
                p{"Only set the fields you want to change, except 'current password', which is required"}
                form method="POST" action={(data.links.http_root)"/userhome?sensitive=1#update-sensitive"} autocomplete="off" {
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


async fn get_render_internal(context: PageContext, 
    update_errors: Option<Vec<String>>, bio_errors: Option<Vec<String>>, private_errors: Option<Vec<String>>) -> Result<Response,Error> 
{
    let private = context.api_context.get_user_private_safe().await;
    let mut userpage : Option<Content> = None;

    if let Some(user) = &context.layout_data.user {
        let mut request = FullRequest::new();
        add_value!(request, "uid", user.id);
        let mut user_request = build_request!(
            RequestType::content, 
            String::from("*"), //ok do we really need it ALL?
            String::from("!userpage(@uid)")
        ); 
        user_request.name = Some(String::from("userpage"));
        request.requests.push(user_request);

        let result = context.api_context.post_request(&request).await?;

        let mut userpage_raw = conversion::cast_result_safe::<Content>(&result, "userpage")?;
        userpage = userpage_raw.pop(); //Doesn't matter if it's none
    }

    Ok(Response::Render(render(context.layout_data, private, userpage, update_errors, bio_errors, private_errors)))
}

pub async fn get_render(context: PageContext) -> Result<Response, Error> {
    get_render_internal(context, None, None, None).await
}


/// Post to update normal info like username, avatar, etc. Note that although this may return an "Error", this is not from
/// having a POST error, it's from a render error for userhome
pub async fn post_info_render(mut context: PageContext, update: UserUpdate) -> Result<Response, Error>
{
    let mut errors = Vec::new();
    //If the user is there, get a copy of it so we can modify and post it
    if let Some(mut current_user) = context.layout_data.user.clone() {
        //Modify
        current_user.username = String::from(update.username);
        current_user.avatar = String::from(update.avatar);
        //Either update the context user or set an error
        match context.api_context.post_userupdate(&current_user).await { 
            Ok(new_user) => context.layout_data.user = Some(new_user), //Update user for rendering
            Err(error) => errors.push(error.to_user_string())
        }
    }
    else {
        errors.push(String::from("Couldn't pull user data, are you still logged in?"));
    }

    get_render_internal(context, Some(errors), None, None).await 
}

/// Complicated function for posting a simple user bio yeesh
pub async fn post_userbio(data: &MainLayoutData, context: &ApiContext, form: &BasicPage) -> Result<Content, Error>
{
    if let Some(ref user) = data.user {
        let mut request = FullRequest::new();
        add_value!(request, "type", "userpages"); //Need the parent

        let mut parent_request = build_request!(
            RequestType::content, 
            String::from("id,parentId,literalType"), 
            String::from("literalType = @type")
        ); 
        parent_request.name = Some(String::from("parent"));
        request.requests.push(parent_request);

        let result = context.post_request(&request).await?;

        let mut parents_raw = conversion::cast_result_required::<Content>(&result, "parent")?;

        match parents_raw.pop() {
            Some(parent) => {
                let mut content = Content::default();
                //note: the hash it autogenerated from the name (hopefully)
                content.text = Some(form.text.clone());
                content.id = Some(form.id);
                content.parentId = parent.id;
                content.contentType = Some(ContentType::USERPAGE);
                content.name = Some(format!("{}'s userpage", user.username));
                content.permissions = Some(make_permissions! {
                    "0": "CR" //Create so people can post on your "wall" (idk if that'll ever happen)
                });
                content.values = Some(make_values! {
                    "markup": "bbcode"
                });
                context.post_content(&content, None).await.map_err(|e| e.into())
            }
            None => {
                Err(Error::Other(String::from("Couldn't find the userpage parent! This is a programming error!")))
            }
        }
    }
    else {
        Err(Error::Other(String::from("Not logged in!")))
    }
}

/// Post to update user bio. It's a bit of a complicated process, but you call this function to perform
/// everything and render the resulting page afterwards, error or not
pub async fn post_bio_render(context: PageContext, bio: BasicPage) -> Result<Response, Error>
{
    //Both go to the same place, AND the userhome renderer reads the data after this write anyway,
    //so you just have to handle the errors
    let mut errors = Vec::new();
    match post_userbio(&context.layout_data, &context.api_context, &bio).await {
        Ok(_content) => {},
        Err(error) => { errors.push(error.to_user_string()) }
    };

    get_render_internal(context, None, Some(errors), None).await 
}

pub async fn post_sensitive_render(context: PageContext, sensitive: UserSensitive) -> Result<Response, Error>
{
    let mut errors = Vec::new();
    match context.api_context.post_usersensitive(&sensitive).await {
        Ok(_token) => {} //Don't need the token
        Err(error) => { errors.push(error.to_user_string()) }
    };

    get_render_internal(context, None, None, Some(errors)).await 
}