use bbscope::BBCode;
use maud::*;
use std::collections::HashMap;

use super::common::contentapi::*;
use super::common::render::*;
use super::common::*;
use super::context::*;
use crate::layout::*;
use crate::links::*;
use crate::response::*;

struct UserPackage {
    pub user: User2,
    pub userpage: Option<BasicContent>,
    pub users: HashMap<i64, User2>,
    pub submissions: Vec<BrowseContent>,
    pub badges: Vec<BrowseContent>,
}

// Find user by name
fn get_user_by_name(ctx: &PageContext, name: &str) -> Result<Option<User2>, Error> {
    let query = format!(
        "SELECT {} FROM users WHERE {} AND username = ?",
        USER2FIELDS, COMMONUSER,
    );
    let params: Vec<&dyn rusqlite::types::ToSql> = vec![&name];
    let mut stmt = ctx.dbcon.prepare(&query)?;

    let mut users = gather_users((&mut stmt, &query), params.as_slice())?;
    Ok(users.pop())
}

fn get_badges(ctx: &PageContext, uid: i64) -> Result<Vec<BrowseContent>, Error> {
    let query = format!(
        "SELECT {} FROM content AS c WHERE {} AND contentType=? AND c.id IN (SELECT relatedId FROM user_relations WHERE userId = ? AND type = ?)",
        BROWSEFIELDS,
        COMMONCONTENT,
    );

    let mut stmt = ctx.dbcon.prepare(&query)?;
    gather_browsecontent(
        (&mut stmt, &query),
        &mut None,
        rusqlite::params![&ContentType::FILE, &uid, &UserRelationType::ASSIGNCONTENT],
    )
}

fn get_userpage(ctx: &PageContext, user: i64) -> Result<Option<BasicContent>, Error> {
    let query = format!(
        "SELECT {} FROM content c {} WHERE c.contentType = ? AND c.createUserId = ? ORDER BY c.id DESC LIMIT 1",
        BASICCONTENTFIELDS,
        join_common_content("c.id")
    );
    let mut stmt = ctx.dbcon.prepare(&query)?;
    Ok(gather_basiccontent(
        (&mut stmt, &query),
        rusqlite::params![ContentType::USERPAGE, &user],
    )?
    .pop())
}

fn render(data: MainLayoutData, mut bbcode: BBCode, user_package: UserPackage) -> String {
    let user = user_package.user;

    let avatar_link = data.links.image(&user.avatar, QueryImage::Cropped300);

    let meta = LayoutMeta {
        title: format!("SBS ⦁ {}", user.username),
        description: String::new(),
        image: Some(avatar_link.clone()),
        canonical: Some(data.links.user_unsafe(&user.username)),
    };

    layout_with_meta(&data, meta, html!{
        (data.links.style("/forpage/user.css"))
        section {
            div #"pageuser" {
                img src={(avatar_link)};
                div #"infoblock" {
                    h1 {(user.username)}
                    div."aside mediumseparate" #"userinfo" {
                        // Some info about the user 
                        div { "Member since: " time { (user.create_date.to_rfc3339()) } }
                        div { "ID: "(user.id) }
                        @if user.admin {
                            div #"adminicon" title="Administrator / Moderator" { "🌟" }
                        }
                    }
                    //If the user has no bio, that's ok! 
                    @if let Some(userpage) = user_package.userpage {
                        div."content" #"userbio" { (PreEscaped(bbcode.parse_profiled_opt(&userpage.text, format!("userpage-{}", userpage.id)))) } 
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
                        (page_card2(&data.links, page, &user_package.users))
                    }
                }
            }
        }
        @if user_package.badges.len() > 0 {
            section {
                h2 { "Legacy badges:" }
                div."badges" {
                    @for ref badge in user_package.badges {
                        img."badge" title=(&badge.name) src=(data.links.image_default(&badge.hash));
                    }
                }
                p."aside" {"Don't worry if you don't have these!"}
            }
        }
    }).into_string()
}

async fn get_render_internal(context: PageContext, username: String) -> Result<Response, Error> {
    let user = get_user_by_name(&context, &username)?;

    if let Some(user) = user {
        let mut users = HashMap::new();
        users.insert(user.id, user.clone());
        let badges = get_badges(&context, user.id)?;

        //OK we did the standard user request. we COULD'VE merged these two, but it's just easier to
        //make a second request for their submissions!
        let mut search = PageSearch::default();
        search.subtype = None; //Don't worry about the type, show ALL submissions
        search.user_id = Some(user.id);
        search.order = "id_desc".to_string(); //not sure...
        let pages = get_browse(&context, &search, QueryLimit::default())?; //get_search_request(&search, 0); //Just ask for as much as possible

        let user_id = user.id;

        let package = UserPackage {
            user,
            userpage: get_userpage(&context, user_id)?, //content_raw.pop(),
            badges,
            submissions: pages,
            users,
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
