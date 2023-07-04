use axum::{
    routing::{get, post},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};

use tower_http::services::{ServeDir, ServeFile};

pub fn get_all_routes() -> Router 
{
    //let fs_static_route = warp::path("static").and(warp::fs::dir("static")).boxed();
    //let fs_favicon_route = warp::path("favicon.ico").and(warp::fs::file("static/resources/favicon.ico")).boxed();
    //let fs_robots_route = warp::path("robots.txt").and(warp::fs::file("static/robots.txt")).boxed();

    // build our application with a route
    let app = Router::new()
        .nest_service("static", ServeDir::new("static"))
        .nest_service("favicon.ico", ServeFile::new("static/resources/favicon.ico"))
        .nest_service("robots.txt", ServeFile::new("static/robots.txt"))
    ;
        // `GET /` goes to `root`
        //.route("/", get(root))
        // `POST /users` goes to `create_user`
        //.route("/users", post(create_user));

    app
}