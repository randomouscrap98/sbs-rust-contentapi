
use std::collections::HashMap;

use bbscope::BBCode;
//use common::constants::SBSPageType;
use contentapi::*; 
use contentapi::forms::*;

use common::*;
use common::forms::*;
use common::render::*;
use common::render::layout::*;
use common::render::submissions::*;
use common::submissions::*;
use maud::*;

pub struct UserPackage {
    pub user: User,
    pub userpage: Option<Content>,
    pub users: HashMap<i64, User>,
    pub submissions: Vec<Content>,
    pub badges: Vec<Content>
}

pub fn render(data: MainLayoutData, mut bbcode: BBCode, user_package: Option<UserPackage>) -> String 
{
    layout(&data, html!{
        (data.links.style("/forpage/user.css"))
        @if let Some(user_package) = user_package {
            @let user = user_package.user;
            section {
                div #"pageuser" {
                    img src={(data.links.image(&user.avatar, &QueryImage::avatar(300)))};
                    div #"infoblock" {
                        h1 {(user.username)}
                        div."aside mediumseparate" #"userinfo" {
                            // Some info about the user 
                            div { "Member since: " time { (user.createDate.to_rfc3339()) } }
                            div { "ID: "(user.id) }
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
        }
        @else {
            section {
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
    let badges_raw = contentapi::conversion::cast_result_required::<Content>(&result, "badges")?;

    let user = users_raw.pop();
    let package: Option<UserPackage> = if let Some(user) = user {
        //OK we did the standard user request. we COULD'VE merged these two, but it's just easier to 
        //make a second request for their submissions!
        let mut search = PageSearch::default();
        search.user_id = Some(user.id);
        let request = get_search_request(&search, 0); //Just ask for as much as possible

        let result = context.api_context.post_request(&request).await?;

        Some(UserPackage {
            user,
            userpage: content_raw.pop(),
            badges: badges_raw,
            submissions: conversion::cast_result_safe::<Content>(&result, "content")?,
            users: conversion::map_users(conversion::cast_result_safe::<User>(&result, "user")?)
        })
    }
    else {
        None
    };

    Ok(Response::Render(render(
        context.layout_data, 
        context.bbcode, 
        package
    )))
}
