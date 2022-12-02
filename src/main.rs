#[macro_use] extern crate rocket;

//use rocket::response::Redirect;
//use rocket::{State, http::Cookie};
//use rocket::form::Form;
use rocket::fairing::AdHoc;
use rocket::fs::FileServer;
//use rocket::http::CookieJar;
use rocket::response::status::Custom as RocketCustom;
use rocket_dyn_templates::Template;

mod config;
mod api;
mod api_data;
mod context;
mod forms;
mod hbs_custom;
mod conversion;
mod routes;

use routes::*;

/* 
    OK so this file should be entirely just routing and the top level stuff required to route. 
    Think of it as all your 'controllers' from ASP.NET put into one. Anything more complicated 
    than rendering a template or basic data parsing should be put in some OTHER function. We 
    have a module 'special_queries' which houses a lot of the complex functionality, and 'api' 
    for general calls to API endpoints.
*/


// ------------------------
// ------- ROUTES ---------
// ------------------------

#[get("/activity")] //this ofc has param values
async fn activity_get(context: context::Context) -> Result<Template, RocketCustom<String>> {
    Ok(basic_template!("activity", context, {}))
}

#[get("/search")] //this ofc has param values
async fn search_get(context: context::Context) -> Result<Template, RocketCustom<String>> {
    Ok(basic_template!("search", context, {}))
}



// -------------------------
// ------- LAUNCH ----------
// -------------------------

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![
            routes::basic::index_get, 
            routes::basic::about_get,
            routes::user::login_get, 
            routes::user::login_post, 
            routes::user::loginrecover_post,
            routes::user::usersensitive_post,
            routes::user::userhome_get, 
            routes::user::userhome_update_post,
            routes::user::logout_get, 
            routes::register::register_get,
            routes::register::register_post,
            routes::register::registerconfirm_get,
            routes::register::registerconfirm_post,
            routes::register::registerresend_post,
            routes::forums::forum_get,
            activity_get,
            search_get,
            routes::imagebrowser::widget_imagebrowser_get,
            routes::imagebrowser::widget_imagebrowser_post
        ])
        .mount("/static", FileServer::from("static/"))
        .manage(context::InitData {
            boot_time : chrono::offset::Utc::now()
        })
        .attach(AdHoc::config::<config::Config>())
        .attach(Template::custom(|engines| {
            hbs_custom::customize(&mut engines.handlebars);
        }))
}
