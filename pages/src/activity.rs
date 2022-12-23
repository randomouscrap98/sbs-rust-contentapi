use chrono::{DateTime, Utc};
use common::*;
use common::layout::*;
use contentapi::*;
use contentapi::conversion::*;
use maud::{html, Markup, PreEscaped};
use serde::{Serialize, Deserialize};

pub static POSTACTIVITYKEY: &str = "post_activity";
pub static USERACTIVITYKEY: &str = "user_activity";
pub static ACTIVITYKEY: &str = "activity";

pub fn render(data: MainLayoutData, activity: Vec<SbsActivity>) -> String {
    layout(&data, html!{
        (style(&data.config, "/forpage/activity.css"))
        section {
            h1 { "Activity" }
        }
        section {
            div."activitylist" {
                @for (index, a) in activity.iter().enumerate() {
                    (activity_item(&data.config, a))
                    @if index < activity.len() - 1 {
                        hr."smaller";
                    }
                }
            }
            div."activitynav" {

            }
        }
    }).into_string()
}

pub fn activity_item(config: &LinkConfig, item: &SbsActivity) -> Markup {
    html!(
        div."activity" {
            div."activityleft" {
                img."avatar" src=(image_link(config, &item.user.avatar, 100, true));
            }
            div."activityright" {
                div."main" {
                    a."username flatlink" href=(user_link(config, item.user)) { (item.user.username) }
                    span { (item.action_text) }
                    @if let Some((href, text)) = &item.activity_href {
                        (activity_link(text, href))
                    }
                    //span."action" { (PreEscaped(&item.action_text)) }
                    time."aside" datetime=(dd(&item.date)) { (timeago(&item.date)) } 
                }
                @if let Some(extra) = &item.extra_text {
                    div."aside extra postpreview" { (PreEscaped(extra)) }
                }
            }
        }
    )
}

pub fn activity_link(text: &str, href: &str) -> Markup {
    html!( a."flatlink" href=(href) { (text) })
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ActivityQuery {
    /// Used when moving forward through activity: the "next" button
    pub start: Option<DateTime<Utc>>,
    /// Used when moving backward through activity: the "previous" button
    pub end: Option<DateTime<Utc>>
}

pub struct SbsActivity<'a> {
    pub date: DateTime<Utc>,
    pub user: &'a User,
    pub action_text: String, //This is RAW, WITH whatever links you need!
    pub activity_href: Option<(String,String)>,
    pub extra_text: Option<String>,
}


/// Returns the constructed query and whether you need to invert the results from your normal order
pub fn get_activity_request(query: &ActivityQuery, per_page: i32) -> FullRequest //(FullRequest, bool)
{
    let mut request = FullRequest::new();
    //let mut inverted = false;

    //Note: the allowed list of types for activity is NOT the same as the allowed list of types for
    //displaying as a thread! We don't want to scare people by putting private threads in the activity
    add_value!(request, "allowed_types", vec![
        SBSContentType::program.to_string(), 
        SBSContentType::resource.to_string(),
        SBSContentType::forumthread.to_string()
    ]); //common::forum::ALLOWEDTYPES);

    let mut user_query = String::new();
    let mut message_query = String::from("!basiccomments() and !literaltypein(@allowed_types)");
    let mut activity_query = String::from("!basichistory() and !literaltypein(@allowed_types)");
    let mut order_cd = "createDate_desc";
    let mut order_d = "date_desc";

    let dq_part = if let Some(start) = query.start {
        add_value!(request, "start", dd(&start));
        //Strictly less than, it's the last date from the previous page
        "< @start"
    }
    else if let Some(end) = query.end {
        add_value!(request, "end", dd(&end));
        //inverted = true; //Need to both invert the queries and the resulting data
        order_cd = "id";
        order_d = "date";
        //Strictly greater than, it's the first date from the next page
        "> @end"
    }
    else {
        ""
    };

    // We ARE limiting by date, go ahead and finish constructing the queries
    if ! dq_part.is_empty() {
        message_query = format!("{} and createDate {}", message_query, dq_part);
        activity_query = format!("{} and date {}", activity_query, dq_part);
        user_query = format!("createDate {}", dq_part);
    }

    let mut user_request = build_request!(
        RequestType::user,
        String::from("*"), //query, order, limit
        user_query,
        order_cd.to_string(),
        per_page
    );
    user_request.name = Some(String::from(USERACTIVITYKEY));
    request.requests.push(user_request);

    let mut message_request = build_request!(
        RequestType::message,
        String::from("*"), //query, order, limit
        message_query,
        order_cd.to_string(),
        per_page
    );
    //message_request.expensive = true;
    message_request.name = Some(String::from(POSTACTIVITYKEY));
    request.requests.push(message_request);

    let mut activity_request = build_request!(
        RequestType::activity,
        String::from("*"), //query, order, limit
        activity_query,
        order_d.to_string(), //Activity has a stupid specially named date field
        per_page
    );
    //activity_request.expensive = true;
    activity_request.name = Some(String::from(ACTIVITYKEY));
    request.requests.push(activity_request);

    let content_request = build_request!(
        RequestType::content,
        String::from("id,name,hash,literalType"), //query, order, limit
        format!("id in @{}.contentId or id in @{}.contentId", POSTACTIVITYKEY, ACTIVITYKEY)
    );
    request.requests.push(content_request);

    let user_request = build_request!(
        RequestType::user,
        String::from("*"), //query, order, limit
        format!("id in @{}.id or id in @{}.createUserId or id in @{}.userId", USERACTIVITYKEY, POSTACTIVITYKEY, ACTIVITYKEY)
    );
    request.requests.push(user_request);

    //(request, inverted)
    request

    //add_value!(request, "");
}

//pub async fn get_activity<'a>(context: &mut PageContext, query: &ActivityQuery, per_page: i32) -> Result<Vec<SbsActivity<'a>>, Error>
//{
//}
macro_rules! getdef {
    ($default:ident,$map:ident,$idfield:expr) => {
        {
            let mut this_thing = &$default;
            if let Some(id) = &$idfield {
                if let Some(item) = &$map.get(id) {
                    this_thing = item;
                }
            }
            this_thing
        }
    };
}

pub async fn get_render(mut context: PageContext, query: ActivityQuery, per_page: i32) -> Result<Response, Error>
{
    //let (request,inverted) = get_activity_request(query, per_page);
    let request = get_activity_request(&query, per_page);
    let response = context.api_context.post_request_profiled_opt(&request, "activity-main").await?;

    let user_activity = cast_result_required::<User>(&response, USERACTIVITYKEY)?;
    let post_activity = cast_result_required::<Message>(&response, POSTACTIVITYKEY)?;
    let content_activity = cast_result_required::<Activity>(&response, ACTIVITYKEY)?;
    let content_raw = cast_result_required::<Content>(&response, "content")?;
    let users_raw = cast_result_required::<User>(&response, "user")?;
    let users = map_users(users_raw);
    let content = map_content(content_raw);

    let mut result : Vec<SbsActivity> = Vec::new();

    for newuser in &user_activity {
        result.push(SbsActivity { 
            date: newuser.createDate, 
            user: newuser, 
            action_text: String::from("created an account!"), 
            activity_href: None,
            extra_text: None
        })
    }

    let default_user = user_or_default(None);
    let default_content = content_or_default(None);

    for post in &post_activity 
    {
        let this_user = getdef!(default_user, users, post.createUserId);
        let this_content = getdef!(default_content, content, post.contentId);
        result.push(SbsActivity { 
            date: post.createDate.unwrap_or_default(), 
            user: this_user,
            action_text: String::from("posted on"), 
            activity_href: Some((forum_post_link(&context.layout_data.config, post, &this_content),String::from(s(&this_content.name)))),
            extra_text: Some(context.bbcode.parse_profiled_opt(s(&post.text), format!("post-{}", i(&post.id))))
        })
    }

    for activity in &content_activity 
    {
        let this_user = getdef!(default_user, users, activity.userId);
        let this_content = getdef!(default_content, content, activity.contentId);

        let action_text = format!("{} {}",
            match activity.action.unwrap_or_else(||0) {
                UserAction::CREATE => "created",
                UserAction::UPDATE => "edited",
                UserAction::DELETE => "deleted",
                _ => "did SOMETHING UNKNOWN(??)"
            },
            {
                //let lit_type = this_content.literalType.as_ref().and_then(|lt| Some(lt.clone())).unwrap_or_else(||String::new());
                if this_content.literalType == Some(SBSContentType::program.to_string()) { "program" }
                else if this_content.literalType == Some(SBSContentType::forumthread.to_string()) { "thread" }
                else if this_content.literalType == Some(SBSContentType::resource.to_string()) { "page" }
                else { "content" }
            }
        );

        result.push(SbsActivity { 
            date: activity.date.unwrap_or_default(), 
            user: this_user,
            action_text, //: String::from("posted on"), 
            activity_href: Some((forum_thread_link(&context.layout_data.config, &this_content),String::from(s(&this_content.name)))),
            extra_text: activity.message.clone()
        })
    }

    result.sort_by(|a, b| b.date.partial_cmp(&a.date).unwrap());

    Ok(Response::Render(render(context.layout_data, result.into_iter().take(per_page as usize).collect())))
}