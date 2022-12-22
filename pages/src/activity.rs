use chrono::{DateTime, Utc};
use common::*;
use common::layout::*;
use contentapi::{FullRequest, build_request, RequestType, add_value};
use maud::html;
use serde::{Serialize, Deserialize};

pub fn render(data: MainLayoutData) -> String {
    layout(&data, html!{
        section {
            h1 {"Activity?? It'll be different"}
        }
    }).into_string()
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ActivityQuery {
    /// Used when moving forward through activity: the "next" button
    pub start: Option<DateTime<Utc>>,
    /// Used when moving backward through activity: the "previous" button
    pub end: Option<DateTime<Utc>>
}

//pub async fn get_activity(context: &mut PageContext, query: &ActivityQuery, per_page: i32)

/// Returns the constructed query and whether you need to invert the results from your normal order
pub async fn get_activity_request(query: &ActivityQuery, per_page: i32) -> (FullRequest, bool)
{
    let mut request = FullRequest::new();
    let mut inverted = false;

    add_value!(request, "allowed_types", common::forum::ALLOWEDTYPES);
    //ONLY one or the other!
    //let date_query: &'static str;
    //let activity_date_query: &'static str;

    let mut user_query = String::new();
    let mut message_query = String::from("!basiccomments() and content_literalType in @allowed_types");
    let mut order_cd = "createDate_desc";
    let mut order_d = "date_desc";

    if let Some(start) = query.start {
        add_value!(request, "start", dd(&start));
        //Strictly less than, it's the last date from the previous page
        let q = "< @start";
        message_query = format!("{} and createDate {}", message_query, q);
        user_query = format!("createDate {}", q);
        //date_query = "createDate < @start";
        //activity_date_query = "date < @start";
    }
    else if let Some(end) = query.end {
        add_value!(request, "end", dd(&end));
        inverted = true; //Need to both invert the queries and the resulting data
        order_cd = "createDate";
        order_d = "date";
        let q = "> @end";
        message_query = format!("{} and createDate {}", message_query, q);
        user_query = format!("createDate {}", q);
        //Strictly greater than, it's the first date from the next page
        //date_query = "createDate > @end";
        //activity_date_query = "date > @end";
    }

    let mut user_request = build_request!(
        RequestType::user,
        String::from("*"), //query, order, limit
        user_query,
        format!("createDate{}", if inverted {""} else {"_desc"}),
        per_page
    );
    user_request.name = Some(String::from("user_activity"));

    let mut message_request = build_request!(
        RequestType::user,
        String::from("*"), //query, order, limit
        message_query,
        format!("createDate{}", if inverted {""} else {"_desc"}),
        per_page
    );
    message_request.expensive = true;
    message_request.name = Some(String::from("post_activity"));

    (request, inverted)

    //add_value!(request, "");
}