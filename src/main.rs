#[macro_use] extern crate rocket;

use rocket::fairing::AdHoc;
use rocket::fs::FileServer;
use rocket_dyn_templates::Template;

// Every extra file gets turned into a module if you define it here.
// These modules can do the same thing for their own children
mod config;
mod api;
mod api_data;
mod context;
mod forms;
mod hbs_custom;
mod conversion;
mod routes;
mod bbcode;

use routes::*;

#[get("/activity")] //this ofc has param values
async fn activity_get(context: context::Context) -> Result<Template, RouteError> {
    Ok(basic_template!("activity", context, {}))
}

#[get("/search")] //this ofc has param values
async fn search_get(context: context::Context) -> Result<Template, RouteError> {
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
            routes::forums::forum_categoryfcid_get,
            routes::forums::forum_categoryhash_get,
            routes::forums::forum_threadhash_get,
            routes::forums::forum_threadhash_postid_get,
            routes::forums::forum_thread_ftid_get,
            routes::forums::forum_thread_fpid_get,
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
