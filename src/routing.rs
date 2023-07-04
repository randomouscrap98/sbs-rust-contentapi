use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router, extract::{State, DefaultBodyLimit, Path},
};

use tower_http::{services::{ServeDir, ServeFile}, limit::RequestBodyLimitLayer};

use crate::state::{RequestContext, GlobalState};

pub fn get_all_routes(gstate: Arc<GlobalState>) -> Router 
{
    // build our application with a route
    let app = Router::new()
        .route("/", get(get_index))
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/favicon.ico", ServeFile::new("static/resources/favicon.ico"))
        .nest_service("/robots.txt", ServeFile::new("static/robots.txt"))
        .with_state(gstate.clone())
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(
            gstate.config.body_maxsize as usize
        ))
    ;
    //let get_index_route = warp_get_async!(
    //    warp::path::end(), 
    //    |context:RequestContext| std_resp!(pages::index::get_render(pc!(context)), context)
    //);

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

pub async fn get_index(State(gstate) : State<Arc<GlobalState>>, path : axum::http::Uri) -> Result<common::response::Response, common::response::Error> //Result<Html<String>, ResponseError>
{
    let context = RequestContext::generate(gstate, &path.to_string(), None, None).await?;
    Ok(pages::index::get_render(context.page_context).await?)
}