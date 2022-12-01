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
mod conversion;
mod special_queries;


macro_rules! my_redirect {
    ($config:expr, $location:expr) => {
        Redirect::to(format!("{}{}", $config.http_root, $location))
    };
}

//Most page rendering is exactly the same and requires the same base data.
//Just simplify that into a macro... (although we could probably do better)
macro_rules! basic_template{
    ($template:expr, $context:ident, {
        $($field_name:ident : $field_value:expr),*$(,)*
    }) => {
        Template::render($template, context! {
            //Only need to borrow everything from context, since it's all 
            //cloned values anyway. Also, this only works because context is passed
            //into the function as a guard, so the lifetime extends beyond the function
            //call and so can be part of the return value (being the template render)
            http_root : &$context.config.http_root,
            http_static : format!("{}/static", &$context.config.http_root),
            http_resources : format!("{}/static/resources", &$context.config.http_root),
            api_fileraw : &$context.config.api_fileraw,
            route_path: &$context.route_path,
            route_uri: &$context.route_path,
            boot_time: &$context.init.boot_time,
            client_ip : &$context.client_ip,
            user: api::get_user_safe(&$context).await,
            api_about: api::get_about(&$context).await.map_err(rocket_error!())?,
            language_code: "en", //Eventually!!
            $($field_name: $field_value,)*
        })
    };
}

macro_rules! userhome_base {
    ($context:ident, { $($uf:ident : $uv:expr),*$(,)* }) => {
        basic_template!("userhome", $context, {
            userprivate : api::get_user_private_safe(&$context).await,
            $($uf: $uv,)*
        })
    };
}

macro_rules! register_base {
    ($context:ident, { $($uf:ident : $uv:expr),*$(,)* }) => {
        basic_template!("register", $context, { $($uf: $uv,)* })
    };
}

macro_rules! registerconfirm_base {
    ($context:ident, { $($uf:ident : $uv:expr),*$(,)* }) => {
        basic_template!("registerconfirm", $context, { $($uf: $uv,)* })
    };
}


macro_rules! login {
    ($jar:ident, $context: ident, $token: expr) => {
        login!($jar, $context, $token, 0) 
    };
    ($jar:ident, $context: ident, $token: expr, $expire: expr) => {
        //Again with the wasting memory and cpu, it's whatever. If we needed THAT much optimization,
        //uhh... well we'd have a lot of other problems than just a single small key copy on the heap
        let mut cookie = Cookie::build($context.config.token_cookie_key.clone(), $token);
        //Here, we say "if you send us an expiration, set the expiration. Otherwise, let it expire
        //at the end of the session"
        if $expire != 0 {
            cookie = cookie.max_age(Duration::seconds($expire as i64));
        }
        $jar.add(cookie.finish());
    };
}

//All our email endpoints in the API just return bools, handling them is a pain, lots of checking and building
macro_rules! handle_email {
    ($email_result:expr, $errors:ident) => {
        handle_error!($email_result, $errors, "Something went wrong sending the email! Try again?");
    };
}

macro_rules! handle_error {
    ($result:expr, $errors:ident) => {
        handle_error!($result, $errors, "Unkown error occurred");
    };
    ($result:expr, $errors:ident, $message:literal) => {
        match $result //post_sendemail(context, email).await
        {
            //If confirmation is successful, we get a token back. We login and redirect to the userhome page
            Ok(success) => {
                if !success {
                    $errors.push(String::from($message));
                }
            },
            //If there's an error, we re-render the confirmation page with the errors.
            Err(error) => {
                $errors.push(error.get_just_string());
            } 
        }
    };
}

//Simple conversion from server error into rocket error. This should ONLY be used where we are certain
//the error isn't due to the user!
macro_rules! rocket_error {
    () => {
        |e| RocketCustom(rocket::http::Status::ServiceUnavailable, e.to_string())
    };
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
    Ok(userhome_base!(context, {}))
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
    Ok(register_base!(context, {}))
}

#[get("/register/confirm")] //This is a PLAIN confirmation page with no extra data
async fn registerconfirm_get(context: context::Context) -> Result<Template, RocketCustom<String>> {
    Ok(registerconfirm_base!(context, { }))
}

#[post("/login", data = "<login>")]
async fn login_post(context: context::Context, login: Form<forms::Login<'_>>, jar: &CookieJar<'_>) -> Result<MultiResponse, RocketCustom<String>> {
    let new_login = conversion::convert_login(&context, &login);
    match api::post_login(&context, &new_login).await
    {
        Ok(result) => {
            login!(jar, context, result, new_login.expireSeconds);
            Ok(MultiResponse::Redirect(my_redirect!(context.config, "/userhome")))
        },
        Err(error) => {
            Ok(MultiResponse::Template(basic_template!("login", context, {errors: vec![error.get_just_string()]})))
        } 
    }
}

#[post("/login?recover", data = "<recover>")]
async fn loginrecover_post(context: context::Context, recover: Form<forms::LoginRecover<'_>>) -> Result<MultiResponse, RocketCustom<String>> {
    let mut errors = Vec::new();
    handle_email!(api::post_email_recover(&context, recover.email).await, errors);
    Ok(MultiResponse::Template(basic_template!("login", context, {
        emailresult : String::from(recover.email), 
        recoversuccess : errors.len() == 0, 
        recovererrors: errors
    })))
}

#[post("/userhome?sensitive", data = "<sensitive>")]
async fn usersensitive_post(context: context::Context, sensitive: Form<forms::UserSensitive<'_>>) -> Result<MultiResponse, RocketCustom<String>> {
    let mut errors = Vec::new();
    handle_error!(api::post_usersensitive(&context, &sensitive).await, errors);
    Ok(MultiResponse::Template(userhome_base!(context, {sensitiveerrors:errors})))
}


#[post("/register", data = "<registration>")]
async fn register_post(context: context::Context, registration: Form<forms::Register<'_>>) -> Result<MultiResponse, RocketCustom<String>> {
    match api::post_register(&context, &registration).await
    {
        //On success, we render the confirmation page with the email result baked in (it's more janky because it's
        //the same page data but on the same route but whatever... it's safer).
        Ok(userresult) => {
            let mut errors = Vec::new();
            //Oh but if the email fails, we need to tell them about it. 
            handle_email!(api::post_email_confirm(&context, registration.email).await, errors);
            //This is the success result registerconfirm render, which should show the user and email. If they
            //navigate away from the page, they'll lose that specialness, but the page will still work if they
            //know their email (why wouldn't they?)
            Ok(MultiResponse::Template(registerconfirm_base!(context, { 
                emailresult : String::from(registration.email),
                userresult : userresult,
                errors: errors
            })))
        },
        //On failure, we re-render the registration page, show errors
        Err(error) => {
            Ok(MultiResponse::Template(register_base!(context, {errors: vec![error.get_just_string()]})))
        } 
    }
}

#[post("/register/confirm", data = "<confirm>")]
async fn registerconfirm_post(context: context::Context, confirm: Form<forms::RegisterConfirm<'_>>, jar: &CookieJar<'_>) -> Result<MultiResponse, RocketCustom<String>> {
    match api::post_registerconfirm(&context, &confirm).await
    {
        //If confirmation is successful, we get a token back. We login and redirect to the userhome page
        Ok(token) => {
            //Registration provides no expiration, so we let the cookie expire as soon as possible
            login!(jar, context, token);
            Ok(MultiResponse::Redirect(my_redirect!(context.config, format!("/userhome"))))
        },
        //If there's an error, we re-render the confirmation page with the errors.
        Err(error) => {
            Ok(MultiResponse::Template(registerconfirm_base!(context, {errors: vec![error.get_just_string()]})))
        } 
    }
}

#[post("/register/confirm?resend", data = "<resendform>")]
async fn registerresend_post(context: context::Context, resendform: Form<forms::RegisterResend<'_>>) -> Result<MultiResponse, RocketCustom<String>> {
    let mut errors = Vec::new();
    handle_email!(api::post_email_confirm(&context, resendform.email).await, errors);
    Ok(MultiResponse::Template(registerconfirm_base!(context, {
        emailresult : String::from(resendform.email), 
        resendsuccess: errors.len() == 0, 
        resenderrors: errors
    })))
}

//Don't need the heavy lifting of an entire context just for logout 
#[get("/logout")]
fn logout_get(config: &State<config::Config>, jar: &CookieJar<'_>) -> Redirect {
    jar.remove(Cookie::named(config.token_cookie_key.clone()));
    my_redirect!(config, "/")
}

#[get("/widget/imagebrowser?<search..>")]
async fn widget_imagebrowser_get(context: context::Context, search: forms::ImageBrowseSearch<'_>) -> Result<Template, RocketCustom<String>> 
{
    let result = special_queries::imagebrowser_request(&context, &search).await.map_err(rocket_error!())?;
    let previews = conversion::cast_result::<api_data::MinimalContent>(&result, "preview").map_err(rocket_error!())?;

    Ok(basic_template!("widgets/imagebrowser", context, {
        search : &search,
        haspreview : previews.len() > 0,
        previewimages : previews,
        imagesize: 100 + 100 * search.size,
        images : conversion::cast_result::<api_data::MinimalContent>(&result, "content").map_err(rocket_error!())?,
        sizevalues : vec![
            hbs_custom::SelectValue::new(1, "1x", search.size), 
            hbs_custom::SelectValue::new(2, "2x", search.size),
            hbs_custom::SelectValue::new(3, "3x", search.size)
        ]
    }))
}

//#[get("/test/request")]
//async fn test_request_get(context: context::Context) -> String {
//    let mut request = api_data::FullRequest::new();
//    request.requests.push(build_request!(api_data::RequestType::user));
//    println!("Sending: {:?}", &request);
//    match api::post_request(&context, &request).await {
//        Ok(result) => {
//            format!("Omg the result is:\n{:?}", result)
//        },
//        Err(error) => {
//            error.get_just_string()
//        }
//    }
//}

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
            loginrecover_post,
            usersensitive_post,
            logout_get, 
            register_get,
            register_post,
            registerconfirm_get,
            registerconfirm_post,
            registerresend_post,
            userhome_get, 
            forum_get,
            activity_get,
            search_get,
            about_get,
            //test_request_get,
            widget_imagebrowser_get
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
