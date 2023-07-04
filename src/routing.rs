use std::sync::Arc;

use axum::{
    routing::{get, post},
    http::StatusCode,
    response::{IntoResponse, Html},
    Json, Router, extract::State,
};

use common::Response;
use tower_http::services::{ServeDir, ServeFile};

use crate::state::RequestContext;

pub fn get_all_routes() -> Router 
{
    // build our application with a route
    let app = Router::new()
        .route("/", get(get_index))
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/favicon.ico", ServeFile::new("static/resources/favicon.ico"))
        .nest_service("/robots.txt", ServeFile::new("static/robots.txt"))
    ;
    //let get_index_route = warp_get_async!(
    //    warp::path::end(), 
    //    |context:RequestContext| std_resp!(pages::index::get_render(pc!(context)), context)
    //);

        // `GET /` goes to `root`
        //.route("/", get(root))
        // `POST /users` goes to `create_user`
        //.route("/users", post(create_user));

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

pub async fn get_index(State(context) : State<Arc<RequestContext>>) -> Response //Result<Html<String>, ResponseError>
{
    Ok(Html(pages::index::get_render(context)))
}