#[macro_use] extern crate rocket;

use rocket::response::Redirect;
use rocket::{State, http::Cookie};
use rocket::form::Form;
use rocket::fairing::AdHoc;
use rocket::fs::FileServer;
use rocket::http::CookieJar;
use rocket::response::status::Custom as RocketCustom;
use rocket_dyn_templates::{Template, context};
use std::net::IpAddr;

mod config;
mod api;
mod api_data;
mod context;
mod forms;
mod hbs_custom;

macro_rules! my_redirect {
    ($config:expr, $location:expr) => {
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
            client_ip : $context.client_ip,
            api_fileraw : $context.config.api_fileraw.clone(),
            user: api::get_user_safe(&$context).await,
            api_about: api::get_about_rocket(&$context).await?,
            $($field_name: $field_value,)*
        })
    }
}

//Generate a basic route with all the stupid cruft we need. Blegh, rocket is cool but it's very
//verbose since it's all through functions. Maybe there's a better way?
macro_rules! basic_route {
    (
        $route_meta:meta, $fn_name:ident ( $($field_name:ident : $field_type:ty),* ), $context:ident => $code:block
    ) => {
        basic_route!{$route_meta, $fn_name($($field_name:$field_type)*), jar, $context => $code}
    };
    (
        $route_meta:meta, $fn_name:ident ( $($field_name:ident : $field_type:ty),* ), $jar:ident, $context:ident => $code:block
    ) => {
        #[$route_meta] //If we need more meta, oh well, fix it later
        async fn $fn_name(config: &State<config::Config>, $jar: &CookieJar<'_>, remote_ip: IpAddr, $($field_name:$field_type)*) -> Result<MultiResponse, RocketCustom<String>> {
            let $context = context::Context::new(config, $jar, remote_ip);
            $code
        }
    };
}

#[derive(Debug, Responder)]
pub enum MultiResponse {
    Template(Template),
    Redirect(Redirect),
}

basic_route!{ get("/"), index_get(), context => {
    Ok(MultiResponse::Template(basic_template!("index", context, {})))
}}

basic_route!{ get("/login"), login_get(), context => {
    Ok(MultiResponse::Template(basic_template!("login", context, {})))
}}

basic_route!{ get("/userhome"), userhome_get(), context => {
    Ok(MultiResponse::Template(basic_template!("userhome", context, {})))
}}

basic_route!{ post("/login", data = "<login>"), login_post(login: Form<forms::Login<'_>>), jar, context => {
    match api::post_login(&context, &login).await
    {
        Ok(result) => {
            //Again with the wasting memory and cpu, it's whatever. If we needed THAT much optimization,
            //uhh... well we'd have a lot of other problems than just a single small key copy on the heap
            jar.add(Cookie::new(context.config.token_cookie_key.clone(), result));
            Ok(MultiResponse::Redirect(my_redirect!(context.config, "/")))
        },
        Err(error) => {
            Ok(MultiResponse::Template(basic_template!("login", context, {errors: vec![error.get_just_string()]})))
        } 
    }
}}

//Don't need the heavy lifting of an entire context just for logout 
#[get("/logout")]
fn logout_get(config: &State<config::Config>, jar: &CookieJar<'_>) -> Redirect {
    jar.remove(Cookie::named(config.token_cookie_key.clone()));
    my_redirect!(config, "/")
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index_get, login_get, login_post, logout_get, userhome_get])
        .mount("/static", FileServer::from("static/"))
        .attach(AdHoc::config::<config::Config>())
        .attach(Template::custom(|engines| {
            hbs_custom::customize(&mut engines.handlebars);
        }))
}
