
use std::collections::HashMap;

use bbscope::BBCode;
use common::prefab::get_documentation_group;
use contentapi::*; 
use contentapi::forms::*;

use common::*;
use common::forms::*;
use common::render::*;
use common::response::*;
use common::render::layout::*;
use common::render::submissions::*;
use common::search::*;
use maud::*;

pub struct UserPackage {
    pub user: User,
    pub userpage: Option<Content>,
    pub users: HashMap<i64, User>,
    pub submissions: Vec<Content>,
    pub badges: Vec<Content>,
    pub docsgroup: User, //docparent: Content,
    pub ban: Option<UserBan>
}

const FULLBAN: i8 = BanType::PRIVATE | BanType::PUBLIC | BanType::USER;
const NORMALBAN: i8 = BanType::PUBLIC;
const PERMABANHOURS: f64 = 24.0 * 365.0 * 100.0;

pub fn render(data: MainLayoutData, mut bbcode: BBCode, user_package: UserPackage, 
    ban_errors: Option<Vec<String>>, unban_errors: Option<Vec<String>>, userset_errors: Option<Vec<String>>) -> String 
{
    let user = user_package.user;

    let meta = LayoutMeta {
        title : format!("SBS ⦁ {}", user.username),
        description : short_description_opt(user_package.userpage.as_ref()),
        image : Some(data.links.image(&user.avatar, &QueryImage::avatar(200))),
        canonical: Some(data.links.user(&user))
    };

    layout_with_meta(&data, meta, html!{
        (data.links.style("/forpage/user.css"))
        section {
            div #"pageuser" {
                img src={(data.links.image(&user.avatar, &QueryImage::avatar(300)))};
                div #"infoblock" {
                    h1 {(user.username)}
                    div."aside mediumseparate" #"userinfo" {
                        // Some info about the user 
                        div { "Member since: " time { (user.createDate.to_rfc3339()) } }
                        div { "ID: "(user.id) }
                        @if user.groups.contains(&user_package.docsgroup.id) {
                            div #"docwritericon" title="Documentation group" { "🕮" }
                        }
                        @if user.admin {
                            div #"adminicon" title="Administrator / Moderator" { "🌟" }
                        }
                    }
                    //If the user has no bio, that's ok! 
                    @if let Some(userpage) = user_package.userpage {
                        div."content" #"userbio" { (PreEscaped(bbcode.parse_profiled_opt(opt_s!(userpage.text), format!("userpage-{}", i(&userpage.id))))) } 
                    }
                }
            }
        }
        section {
            h1 { "Submissions:" }
            @if user_package.submissions.len() == 0 {
                p."aside" { "None!" }
            }
            @else {
                div."cardslist" {
                    @for page in &user_package.submissions {
                        (page_card(&data.links, page, &user_package.users))
                    }
                }
            }
        }
        @if user_package.badges.len() > 0 {
            section {
                h2 { "Legacy badges:" }
                div."badges" {
                    @for ref badge in user_package.badges {
                        img."badge" title=(opt_s!(badge.name)) src=(data.links.image_default(opt_s!(badge.hash)));
                    }
                }
                p."aside" {"Don't worry if you don't have these!"}
            }
        }
        @if let Some(current_user) = &data.user {
            @if current_user.admin {
                section #"admincontrols" {
                    h3 { "Ban controls:" }
                    @if let Some(ban) = &user_package.ban {
                        form #"unbanform" method="POST" action={(data.links.user(&user))"?unban=1#admincontrols"} {
                            (errorlist(unban_errors))
                            p."error" { 
                                "ALREADY" 
                                @if (ban.r#type & FULLBAN) == FULLBAN { b { " FULL " } }
                                @else { " " }
                                "BANNED for: "  
                                time datetime=(dd(&ban.expireDate)) { (timeago_future(&ban.expireDate)) }
                                " - " (opt_s!(ban.message))
                            }
                            label for="unban_reason"{"Unban Reason (for admin logs):"}
                            input."largeinput" #"unban_reason" type="text" required="" name="new_reason";
                            input type="hidden" name="id" value=(ban.id);
                            input type="submit" value="Unban";
                        }
                    }
                    @else {
                        form #"banform" method="POST" action={(data.links.user(&user))"?ban=1#admincontrols"} {
                            (errorlist(ban_errors))
                            label for="ban_hours"{"Ban hours:"}
                            input #"ban_hours" type="text" required="" name="hours" placeholder="0 = 100 years";
                            label for="ban_reason"{"Ban Reason (shown to user):"}
                            input."largeinput" #"ban_reason" type="text" required="" name="reason";
                            div."smallseparate inline" {
                                label for="ban_full" { "FULL ban (blocks from user modifications and dms): " }
                                input #"ban_full" type="checkbox" name="full" value="true"; 
                            }
                            input type="hidden" name="user_id" value=(user.id);
                            input type="submit" value="Ban";
                        }
                    }
                    hr;
                    h3 #"update-user" {"Update user info:"}
                    form method="POST" action={(data.links.user(&user))"?userinfo=1#update-user"} { 
                        p."aside" { 
                            "You can override a user's username and avatar here. Note that they'll be able to change it back "
                            "by default unless you 'full ban' them. So, this form is only useful when full banning a user."
                        }
                        (errorlist(userset_errors))
                        label for="update_username"{"Username:"}
                        input #"update_username" type="text" name="username" value=(user.username);
                        label for="update_avatar"{"Avatar:"}
                        input #"update_avatar" type="text" name="avatar" value=(user.avatar);
                        input type="submit" value="Update";
                    }
                }
            }
        }
    }).into_string()
}


pub async fn get_render_internal(mut context: PageContext, username: String, ban_errors: Option<Vec<String>>,
    unban_errors: Option<Vec<String>>, userset_errors: Option<Vec<String>>) -> Result<Response, Error>
{
    //Go get the user and their userpage
    let mut request = FullRequest::new();
    add_value!(request, "username", username);
    add_value!(request, "relationtype", UserRelationType::ASSIGNCONTENT);
    add_value!(request, "file", ContentType::FILE);

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

    request.requests.push(build_request!(
        RequestType::userrelation, 
        String::from("*"), //ok do we really need it ALL?
        String::from("userId = @user.id AND type = @relationtype") //Unfortunately, we don't do anything else with assigned content in sbs
    )); 
    
    request.requests.push(build_request!(
        RequestType::ban, 
        String::from("*"),
        String::from("bannedUserId = @user.id and !activebans()")
    )); 

    let mut badge_request = build_request!(
        RequestType::content, 
        String::from("id,name,description,contentType,hash,literalType"), 
        String::from("id in @userrelation.relatedId and contentType = @file") //Unfortunately, we don't do anything else with assigned content in sbs
    ); 
    badge_request.name = Some(String::from("badges"));
    request.requests.push(badge_request);

    let result = context.api_context.post_request(&request).await?;

    //Now try to parse two things out of it
    let mut users_raw = contentapi::conversion::cast_result_required::<User>(&result, "user")?;
    let mut content_raw = contentapi::conversion::cast_result_required::<Content>(&result, "content")?;
    let mut bans_raw = contentapi::conversion::cast_result_required::<UserBan>(&result, "ban")?;
    let badges_raw = contentapi::conversion::cast_result_required::<Content>(&result, "badges")?;

    let user = users_raw.pop();
    
    if let Some(user) = user {
        //OK we did the standard user request. we COULD'VE merged these two, but it's just easier to 
        //make a second request for their submissions!
        let mut search = PageSearch::default();
        search.subtype = None; //Don't worry about the type, show ALL submissions
        search.user_id = Some(user.id);
        search.order = "id_desc".to_string(); //not sure...
        let request = get_search_request(&search, 0); //Just ask for as much as possible

        let result = context.api_context.post_request(&request).await?;
        let docsgroup = get_documentation_group(&mut context.api_context).await?;
        //let docparent = get_documentation_parent(&mut context.api_context, DOCPARENTMINIMALFIELDS).await?;

        let package = UserPackage {
            user,
            userpage: content_raw.pop(),
            badges: badges_raw,
            ban: bans_raw.pop(),
            submissions: conversion::cast_result_safe::<Content>(&result, "content")?,
            users: common::view::map_users(conversion::cast_result_safe::<User>(&result, "user")?),
            docsgroup
            //docparent
        };

        Ok(Response::Render(render(
            context.layout_data, 
            context.bbcode, 
            package,
            ban_errors,
            unban_errors,
            userset_errors
        )))
    }
    else {
        Err(Error::NotFound(String::from("User not found!")))
    }

}

pub async fn get_render(context: PageContext, username: String) -> Result<Response, Error>
{
    get_render_internal(context, username, None, None, None).await
}

pub async fn post_ban(context: PageContext, username: String, mut ban: BanForm) -> Result<Response, Error>
{
    let mut errors = Vec::new();

    if ban.hours <= 0.0 { ban.hours = PERMABANHOURS; }

    if let Some(user) = &context.layout_data.user 
    {
        //Need to convert banform to real ban
        let real_ban = UserBan {
            id: 0,
            bannedUserId: ban.user_id,
            createUserId: user.id,
            createDate: chrono::Utc::now(),
            // Users can give arbitrary precision floats for "hours"; we want to capture as much of that as possible by
            // converting it to whole-number milliseconds (Duration only takes i64)
            expireDate: chrono::Utc::now() + chrono::Duration::milliseconds((ban.hours * 60f64 * 60f64 * 1000f64) as i64),
            message: Some(ban.reason),
            r#type: if ban.full { FULLBAN } else { NORMALBAN } 
        };

        match context.api_context.post_ban(&real_ban).await {
            Ok(_token) => {} //Don't need the token
            Err(error) => { errors.push(error.to_user_string()) }
        };
    }
    else {
        errors.push("Must be logged in to ban users!".to_string());
    }

    get_render_internal(context, username, Some(errors), None, None).await
}


pub async fn post_unban(context: PageContext, username: String, unban: UnbanForm) -> Result<Response, Error>
{
    let mut errors = Vec::new();

    //Go get the old ban
    let mut request = FullRequest::new();
    add_value!(request, "id", unban.id);
    request.requests.push(build_request!(
        RequestType::ban, 
        String::from("*"),
        String::from("id = @id and !activebans()")
    )); 

    let result = context.api_context.post_request(&request).await?;
    let mut bans_raw = contentapi::conversion::cast_result_required::<UserBan>(&result, "ban")?;

    if let Some(mut ban) = bans_raw.pop() {
        ban.expireDate = ban.createDate; // ALWAYS in the past
        ban.message = Some(format!("{} - EDIT (UNBANNED): {}", opt_s!(ban.message), unban.new_reason));

        match context.api_context.post_ban(&ban).await {
            Ok(_token) => {} //Don't need the token
            Err(error) => { errors.push(error.to_user_string()) }
        };
    }
    else {
        errors.push(format!("Couldn't find ban with id {}", unban.id));
    }

    get_render_internal(context, username, None, Some(errors), None).await
}

/// Endpoint for admins to post user info updates for users. Note that if normal users attempt to use this endpoint,
/// contentapi should reject them since you're not allowed to change other user's junk
pub async fn post_userinfo(context: PageContext, username: String, update: UserUpdate) -> Result<Response, Error>
{
    let mut errors = Vec::new();

    //Go find the user. Remember you have to use SPECIFICALLY the username from the route, NOT from the form!
    match context.api_context.get_user_by_username(&username, "*").await {
        Ok(mut user) => {
            user.username = update.username.clone(); 
            user.avatar = update.avatar.clone();
            match context.api_context.post_userupdate(&user).await { 
                Ok(new_user) => { 
                    return Ok(Response::Redirect(context.layout_data.links.user(&new_user)))
                }, //Return a redirect to the NEW user
                Err(error) => errors.push(error.to_user_string())
            }
        },
        Err(error) => {
            errors.push(error.to_user_string());
        }
    }

    //If you get here, it's almost certainly an error
    get_render_internal(context, username, None, None, Some(errors)).await
}