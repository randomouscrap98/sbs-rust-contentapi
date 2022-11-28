#[macro_use] extern crate rocket;

use rocket::response::Redirect;
use rocket::{State, http::Cookie};
use rocket::form::Form;
use rocket::fairing::AdHoc;
use rocket::fs::FileServer;
use rocket::http::CookieJar;
use rocket::response::status::Custom as RocketCustom;
use rocket_dyn_templates::{Template, context};

mod config;
mod api;
mod api_data;
mod context;
mod forms;
mod hbs_custom;

macro_rules! my_redirect {
    ($config:ident, $location:expr) => {
        Redirect::to(format!("{}{}", $config.http_root, $location))
    };
}

//Most page rendering is exactly the same and requires the same base data.
//Just simplify that into a macro... (although we could probably do better)
macro_rules! basic_template{
    ($template:expr, $context:ident, {
        $($field_name:ident : $field_value:expr),*
    }) => {
        Template::render($template, context! {
            http_root : $context.config.http_root.clone(),
            http_static : format!("{}/static", &$context.config.http_root),
            http_resources : format!("{}/static/resources", &$context.config.http_root),
            api_fileraw : $context.config.api_fileraw.clone(),
            user: api::get_user_safe(&$context).await,
            api_about: api::get_about_rocket(&$context).await?,
            $($field_name: $field_value,)*
        })
    }
}

#[derive(Debug, Responder)]
pub enum MultiResponse {
    Template(Template),
    Redirect(Redirect),
}

#[get("/")]
async fn index_get(config: &State<config::Config>, jar: &CookieJar<'_>) -> Result<Template, RocketCustom<String>> {
    let context = context::Context::new(config, jar);
    Ok(basic_template!("index", context, { }))
}

#[get("/login")]
async fn login_get(config: &State<config::Config>, jar: &CookieJar<'_>) -> Result<Template, RocketCustom<String>> {
    let context = context::Context::new(config, jar);
    Ok(basic_template!("login", context, {}))
}

#[post("/login", data = "<login>")]
async fn login_post(login: Form<forms::Login<'_>>, config: &State<config::Config>, jar: &CookieJar<'_>) -> Result<MultiResponse, RocketCustom<String>> {
    let context = context::Context::new(config, jar);
    match api::post_login(&context, &login).await
    {
        Ok(result) => {
            //Again with the wasting memory and cpu, it's whatever. If we needed THAT much optimization,
            //uhh... well we'd have a lot of other problems than just a single small key copy on the heap
            jar.add(Cookie::new(config.token_cookie_key.clone(), result));
            Ok(MultiResponse::Redirect(my_redirect!(config, "/")))
        },
        Err(error) => {
            Ok(MultiResponse::Template(basic_template!("login", context, {errors: vec![error.get_just_string()]})))
        } 
    }
}

#[get("/logout")]
fn logout_get(config: &State<config::Config>, jar: &CookieJar<'_>) -> Redirect {
    jar.remove(Cookie::named(config.token_cookie_key.clone()));
    my_redirect!(config, "/")
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index_get, login_get, login_post, logout_get])
        .mount("/static", FileServer::from("static/"))
        .attach(AdHoc::config::<config::Config>())
        .attach(Template::custom(|engines| {
            hbs_custom::customize(&mut engines.handlebars);
        }))
}
