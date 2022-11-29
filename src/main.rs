#[macro_use] extern crate rocket;

use rocket::response::Redirect;
use rocket::time::Duration;
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
            route_path: $context.route_path,
            $($field_name: $field_value,)*
        })
    }
}

#[derive(Debug, Responder)]
pub enum MultiResponse {
    Template(Template),
    Redirect(Redirect),
}

// ------------------------
// ------- ROUTES ---------
// ------------------------

#[get("/")]
async fn index_get(context: context::Context) -> Result<Template, RocketCustom<String>> {
    Ok(basic_template!("index", context, {}))
}

#[get("/login")]
async fn login_get(context: context::Context) -> Result<Template, RocketCustom<String>> {
    Ok(basic_template!("login", context, {}))
}

#[get("/userhome")]
async fn userhome_get(context: context::Context) -> Result<Template, RocketCustom<String>> {
    Ok(basic_template!("userhome", context, {}))
}

#[get("/forum")]
async fn forum_get(context: context::Context) -> Result<Template, RocketCustom<String>> {
    Ok(basic_template!("forum", context, {}))
}

#[get("/activity")] //this ofc has param values
async fn activity_get(context: context::Context) -> Result<Template, RocketCustom<String>> {
    Ok(basic_template!("activity", context, {}))
}

#[get("/search")] //this ofc has param values
async fn search_get(context: context::Context) -> Result<Template, RocketCustom<String>> {
    Ok(basic_template!("search", context, {}))
}

#[get("/about")] 
async fn about_get(context: context::Context) -> Result<Template, RocketCustom<String>> {
    Ok(basic_template!("about", context, {}))
}

#[get("/register")] 
async fn register_get(context: context::Context) -> Result<Template, RocketCustom<String>> {
    Ok(basic_template!("register", context, {}))
}

#[get("/register/confirm")] 
async fn registerconfirm_get(context: context::Context, jar: &CookieJar<'_>) -> Result<Template, RocketCustom<String>> {
    Ok(basic_template!("registerconfirm", context, { 
        emailresult : jar.get(&context.config.register_cookie_key).and_then(|email| Some(String::from(email.value())))
    }))
}

#[post("/login", data = "<login>")]
async fn login_post(context: context::Context, login: Form<forms::Login<'_>>, jar: &CookieJar<'_>) -> Result<MultiResponse, RocketCustom<String>> {
    match api::post_login(&context, &login).await
    {
        Ok(result) => {
            //Again with the wasting memory and cpu, it's whatever. If we needed THAT much optimization,
            //uhh... well we'd have a lot of other problems than just a single small key copy on the heap
            jar.add(
                Cookie::build(context.config.token_cookie_key.clone(), result)
                    .max_age(Duration::days(context.config.default_token_expire.into()))
                    .finish()
            );
            Ok(MultiResponse::Redirect(my_redirect!(context.config, "/")))
        },
        Err(error) => {
            Ok(MultiResponse::Template(basic_template!("login", context, {errors: vec![error.get_just_string()]})))
        } 
    }
}

#[post("/register", data = "<registration>")]
async fn register_post(context: context::Context, registration: Form<forms::Register<'_>>, jar: &CookieJar<'_>) -> Result<MultiResponse, RocketCustom<String>> {
    match api::post_register(&context, &registration).await
    {
        Ok(_) => {
            //It's fine to render this page at the same url I think? I don't know
            //This is a scary temporary cookie that exposes the user's email. It will be deleted upon registration
            //but... mmm I don't know.
            //jar.add(
            //    Cookie::build(context.config.register_cookie_key.clone(), String::from(registration.email))
            //        .max_age(Duration::minutes(30))
            //        .finish()
            //);
            Ok(MultiResponse::Template(basic_template!("registerconfirm", context, { emailresult : String::from(registration.email)})))
            //Ok(MultiResponse::Redirect(my_redirect!(context.config, format!("/register/confirm"))))
        },
        Err(error) => {
            Ok(MultiResponse::Template(basic_template!("register", context, {errors: vec![error.get_just_string()]})))
        } 
    }
}

#[post("/register/confirm", data = "<confirm>")]
async fn registerconfirm_post(context: context::Context, confirm: Form<forms::RegisterConfirm<'_>>, jar: &CookieJar<'_>) -> Result<MultiResponse, RocketCustom<String>> {
    match api::post_register(&context, &registration).await
    {
        Ok(_) => {
            //This is a scary temporary cookie that exposes the user's email. It will be deleted upon registration
            //but... mmm I don't know.
            jar.add(
                Cookie::build(context.config.register_cookie_key.clone(), String::from(registration.email))
                    .max_age(Duration::minutes(30))
                    .finish()
            );
            Ok(MultiResponse::Redirect(my_redirect!(context.config, format!("/register/confirm"))))
        },
        Err(error) => {
            Ok(MultiResponse::Template(basic_template!("register", context, {errors: vec![error.get_just_string()]})))
        } 
    }
}

//Don't need the heavy lifting of an entire context just for logout 
#[get("/logout")]
fn logout_get(config: &State<config::Config>, jar: &CookieJar<'_>) -> Redirect {
    jar.remove(Cookie::named(config.token_cookie_key.clone()));
    my_redirect!(config, "/")
}

// -------------------------
// ------- LAUNCH ----------
// -------------------------

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![
            index_get, 
            login_get, 
            login_post, 
            logout_get, 
            userhome_get, 
            forum_get,
            activity_get,
            search_get,
            register_get,
            registerconfirm_get,
            register_post,
            registerconfirm_post,
            about_get
        ])
        .mount("/static", FileServer::from("static/"))
        .attach(AdHoc::config::<config::Config>())
        .attach(Template::custom(|engines| {
            hbs_custom::customize(&mut engines.handlebars);
        }))
}
