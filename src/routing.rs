use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router, extract::{State, DefaultBodyLimit, Path, Query, FromRequestParts}, async_trait, Form, 
};

use tower_cookies::{CookieManagerLayer, Cookies};
use tower_http::{services::{ServeDir, ServeFile}, limit::RequestBodyLimitLayer};

use crate::state::{RequestContext, GlobalState};

static SESSIONCOOKIE: &str = "sbs-rust-contentapi-session";
static SETTINGSCOOKIE: &str = "sbs-rust-contentapi-settings";

type StdResponse = Result<common::response::Response, common::response::Error>;

pub fn get_all_routes(gstate: Arc<GlobalState>) -> Router 
{
    // build our application with a route
    let app = Router::new()
        .route("/", 
            get(|context: RequestContext| 
                async { StdResponse::Ok(pages::index::get_render(context.page_context).await?) }))
        .route("/about", 
            get(|context: RequestContext| 
                async { StdResponse::Ok(pages::about::get_render(context.page_context).await?)}))
        .route("/integrationtest", 
            get(|context: RequestContext| 
                async { StdResponse::Ok(pages::integrationtest::get_render(context.page_context).await?) }))
        .route("/documentation", 
            get(|context: RequestContext| 
                async { StdResponse::Ok(pages::documentation::get_render(context.page_context).await?) }))
        .route("/allsearch", 
            get(|context: RequestContext, Query(search): Query<pages::searchall::SearchAllForm>| 
                async { StdResponse::Ok(pages::searchall::get_render(context.page_context, search).await?) }))
        .route("/widget/bbcodepreview", 
            get(|context: RequestContext| 
                async { StdResponse::Ok(pages::widget_bbcodepreview::get_render(context.page_context).await?) })
            .post(|context: RequestContext, Form(form) : Form<common::forms::BasicText>|
                async { StdResponse::Ok(pages::widget_bbcodepreview::post_render(context.page_context, form.text).await?)}))
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/favicon.ico", ServeFile::new("static/resources/favicon.ico"))
        .nest_service("/robots.txt", ServeFile::new("static/robots.txt"))
        .with_state(gstate.clone())
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(
            gstate.config.body_maxsize as usize
        ))
        .layer(CookieManagerLayer::new())
    ;

    //let get_bbcodepreview_route = warp_get!(warp::path!("widget" / "bbcodepreview"),
    //    |context:RequestContext| warp::reply::html(pages::widget_bbcodepreview::render(pc!(context.layout_data), &gs!(context.bbcode), None)));


    //let post_bbcodepreview_route = warp::post()
    //    .and(warp::path!("widget" / "bbcodepreview"))
    //    .and(form_filter.clone())
    //    .and(warp::body::form::<common::forms::BasicText>())
    //    .and(state_filter.clone())
    //    .map(|form: common::forms::BasicText, context: RequestContext| {
    //        warp::reply::html(pages::widget_bbcodepreview::render(context.page_context.layout_data, &context.global_state.bbcode, Some(form.text)))
    //    })
    //    .boxed();

    app
}

/*
    Issues:
    ---------
    * Pages seem to return both a response and an error, when the response usually indicates the entire error? Could maybe flatten
      into just "response" and make a way to produce the appropriate output? Are there any routes that might have errors before
      you get to the rendering? Maybe for the special routes that are either/or... let's assume Response is always enough though.
    * 
 */


#[async_trait]
impl FromRequestParts<Arc<GlobalState>> for RequestContext
//where
    //S: AsRef<Arc<GlobalState>>,
{
    type Rejection = common::response::Error;

    async fn from_request_parts(parts: &mut axum::http::request::Parts, state: &Arc<GlobalState>) -> Result<Self, Self::Rejection>
    {
        use axum::RequestPartsExt;
        let cookies = parts.extract::<Cookies>()
            .await
            .map_err(|err| Self::Rejection::Other(err.1.to_string()))?;
        let path = parts.extract::<axum::http::Uri>()
            .await.unwrap(); //Infallible?

        let token = cookies.get(SESSIONCOOKIE).and_then(|t| Some(t.value().to_string()));
        let config_raw = cookies.get(SETTINGSCOOKIE).and_then(|c| Some(c.value().to_string()));
        RequestContext::generate(state.clone(), &path.to_string(), token, config_raw).await
    }
}
