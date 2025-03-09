use std::collections::HashMap;

use bbscope::BBCode;
use common::prefab::get_documentation_group;
use contentapi::forms::*;
use contentapi::*;

use common::forms::*;
use common::render::layout::*;
use common::render::submissions::*;
use common::render::*;
use common::response::*;
use common::search::*;
use common::*;
use maud::*;

pub struct UserPackage {
    pub user: User,
    pub userpage: Option<Content>,
    pub users: HashMap<i64, User>,
    pub submissions: Vec<Content>,
    pub badges: Vec<Content>,
    pub docsgroup: User, //docparent: Content,
}

pub fn render(data: MainLayoutData, mut bbcode: BBCode, user_package: UserPackage) -> String {
    let user = user_package.user;

    let meta = LayoutMeta {
        title: format!("SBS ⦁ {}", user.username),
        description: short_description_opt(user_package.userpage.as_ref()),
        image: Some(data.links.image(&user.avatar, &QueryImage::avatar(200))),
        canonical: Some(data.links.user(&user)),
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
    }).into_string()
}

pub async fn get_render_internal(
    mut context: PageContext,
    username: String,
) -> Result<Response, Error> {
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
    let mut content_raw =
        contentapi::conversion::cast_result_required::<Content>(&result, "content")?;
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
            submissions: conversion::cast_result_safe::<Content>(&result, "content")?,
            users: common::view::map_users(conversion::cast_result_safe::<User>(&result, "user")?),
            docsgroup, //docparent
        };

        Ok(Response::Render(render(
            context.layout_data,
            context.bbcode,
            package,
        )))
    } else {
        Err(Error::NotFound(String::from("User not found!")))
    }
}

pub async fn get_render(context: PageContext, username: String) -> Result<Response, Error> {
    get_render_internal(context, username).await
}
