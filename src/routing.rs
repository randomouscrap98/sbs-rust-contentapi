use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router, extract::{State, DefaultBodyLimit, Path, Query, FromRequestParts}, async_trait, Extension, middleware::from_extractor_with_state,
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
            get(|context: RequestContext| async { StdResponse::Ok(pages::index::get_render(context.page_context).await?) }))
        .route("/about", 
            get(|context: RequestContext| async { StdResponse::Ok(pages::about::get_render(context.page_context).await?)}))
        .route("/integrationtest", 
            get(|context: RequestContext| async { StdResponse::Ok(pages::integrationtest::get_render(context.page_context).await?) }))
        .route("/documentation", 
            get(|context: RequestContext| async { StdResponse::Ok(pages::documentation::get_render(context.page_context).await?) }))
        .route("/allsearch", get(get_allsearch))
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

            //std_get!(context, pages::index::get_render(context.page_context).await?)))
            //std_get!(context, pages::about::get_render(context.page_context).await?)))

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
where
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

        //let whatever = State::<Arc<GlobalState>>::from_request_parts(parts, state); //from_request_parts(parts).await; //from_extractor_with_state(state).await;
        //let whatever = State::<Arc<GlobalState>>::from_request_parts(parts, state)//from_request_parts(parts).await; //from_extractor_with_state(state).await;
        //TypedHeader::<Authorization<Bearer>>::
        //use axum::RequestPartsExt;
        //let State(state) = parts.extract_with_state::<Arc<GlobalState>, _>(state)
        //    .await
        //    .map_err(|err| err.into_response())?;
    }
}

async fn get_request_context(gstate: Arc<GlobalState>, path: axum::http::Uri, cookies: Cookies) -> Result<RequestContext, common::response::Error>
{
    let token = cookies.get(SESSIONCOOKIE).and_then(|t| Some(t.value().to_string()));
    let config_raw = cookies.get(SETTINGSCOOKIE).and_then(|c| Some(c.value().to_string()));
    RequestContext::generate(gstate, &path.to_string(), token, config_raw).await
}

//macro_rules! std_get {
//    ($ctx:ident, $render:expr) => {
//        |State(gstate) : State<Arc<GlobalState>>, path : axum::http::Uri, cookies: Cookies| async {
//            let $ctx = get_request_context(gstate, path, cookies).await?;
//            Ok::<common::response::Response, common::response::Error>($render) //pages::index::get_render(.page_context).await?)
//        }
//
//    };
//}
//pub(crate) use std_get;


async fn get_index(context: RequestContext) -> StdResponse
{
    Ok(pages::index::get_render(context.page_context).await?)
}

async fn get_about(context: RequestContext) -> StdResponse
{
    Ok(pages::about::get_render(context.page_context).await?)
}

async fn get_integrationtest(context: RequestContext) -> StdResponse
{
    Ok(pages::integrationtest::get_render(context.page_context).await?)
}

async fn get_documentation(context: RequestContext) -> StdResponse
{
    Ok(pages::documentation::get_render(context.page_context).await?)
}

async fn get_allsearch(Query(search) : Query<pages::searchall::SearchAllForm>,  State(gstate) : State<Arc<GlobalState>>, path : axum::http::Uri, cookies: Cookies) -> StdResponse
{
    let context = get_request_context(gstate, path, cookies).await?;
    Ok(pages::searchall::get_render(context.page_context, search).await?)
}