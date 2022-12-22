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
                @for a in &activity {
                    (activity_item(&data.config, a))
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
            div."main" {
                img src=(image_link(config, &item.user.avatar, 100, true));
                a."username" href=(user_link(config, item.user)) { (item.user.username) }
                span."action" { (PreEscaped(&item.action_text)) }
            }
            @if let Some(extra) = &item.extra_text {
                div."extra" { (PreEscaped(extra)) }
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
    pub extra_text: Option<String>,
}


/// Returns the constructed query and whether you need to invert the results from your normal order
pub fn get_activity_request(query: &ActivityQuery, per_page: i32) -> FullRequest //(FullRequest, bool)
{
    let mut request = FullRequest::new();
    //let mut inverted = false;

    add_value!(request, "allowed_types", common::forum::ALLOWEDTYPES);

    let mut user_query = String::new();
    let mut message_query = String::from("!basiccomments() and content_literalType in @allowed_types");
    let mut activity_query = String::from("!basichistory() and literalType in @allowed_types");
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
        order_cd = "createDate";
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
        RequestType::user,
        String::from("*"), //query, order, limit
        message_query,
        order_cd.to_string(),
        per_page
    );
    message_request.expensive = true;
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

    let user_request = build_request!(
        RequestType::user,
        String::from("*"), //query, order, limit
        format!("id in {}.id or id in {}.createUserId or id in {}.userId", USERACTIVITYKEY, POSTACTIVITYKEY, ACTIVITYKEY)
    );
    request.requests.push(user_request);

    //(request, inverted)
    request

    //add_value!(request, "");
}

//pub async fn get_activity<'a>(context: &mut PageContext, query: &ActivityQuery, per_page: i32) -> Result<Vec<SbsActivity<'a>>, Error>
//{
//}

pub async fn get_render(mut context: PageContext, query: ActivityQuery, per_page: i32) -> Result<Response, Error>
{
    //let (request,inverted) = get_activity_request(query, per_page);
    let request = get_activity_request(&query, per_page);
    let response = context.api_context.post_request_profiled_opt(&request, "activity-main").await?;

    let user_activity = cast_result_required::<User>(&response, USERACTIVITYKEY)?;
    let post_activity = cast_result_required::<Message>(&response, POSTACTIVITYKEY)?;
    let content_activity = cast_result_required::<Message>(&response, ACTIVITYKEY)?;
    let users_raw = cast_result_required::<User>(&response, "user")?;
    let users = map_users(users_raw);

    let result : Vec<SbsActivity> = Vec::new();

    Ok(Response::Render(render(context.layout_data, result)))
}