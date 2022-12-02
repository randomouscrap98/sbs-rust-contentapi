use crate::context::*;
use crate::forms;
use crate::conversion;
use crate::config;
//use crate::api_data::*;
use crate::api::*;
use super::*;

use rocket::http::{CookieJar, Cookie};
use rocket::form::Form;
use rocket::response::Redirect;
use rocket::response::status::Custom as RocketCustom;
use rocket_dyn_templates::Template;

macro_rules! userhome_base {
    ($context:ident, { $($uf:ident : $uv:expr),*$(,)* }) => {
        basic_template!("userhome", $context, {
            userprivate : crate::api::get_user_private_safe(&$context).await,
            $($uf: $uv,)*
        })
    };
}
pub(crate) use userhome_base;

#[get("/login")]
pub async fn login_get(context: Context) -> Result<Template, RocketCustom<String>> {
    Ok(basic_template!("login", context, {}))
}

#[get("/userhome")]
pub async fn userhome_get(context: Context) -> Result<Template, RocketCustom<String>> {
    Ok(userhome_base!(context, {}))
}

#[post("/login", data = "<login>")]
pub async fn login_post(context: Context, login: Form<forms::Login<'_>>, jar: &CookieJar<'_>) -> Result<MultiResponse, RocketCustom<String>> {
    let new_login = conversion::convert_login(&context, &login);
    match post_login(&context, &new_login).await
    {
        Ok(result) => {
            login!(jar, context, result, new_login.expireSeconds);
            Ok(MultiResponse::Redirect(my_redirect!(context.config, "/userhome")))
        },
        Err(error) => {
            Ok(MultiResponse::Template(basic_template!("login", context, {errors: vec![error.to_string()]})))
        } 
    }
}

#[post("/login?recover", data = "<recover>")]
pub async fn loginrecover_post(context: Context, recover: Form<forms::LoginRecover<'_>>) -> Result<MultiResponse, RocketCustom<String>> {
    let mut errors = Vec::new();
    handle_email!(post_email_recover(&context, recover.email).await, errors);
    Ok(MultiResponse::Template(basic_template!("login", context, {
        emailresult : String::from(recover.email), 
        recoversuccess : errors.len() == 0, 
        recovererrors: errors
    })))
}

#[post("/userhome?sensitive", data = "<sensitive>")]
pub async fn usersensitive_post(context: Context, sensitive: Form<forms::UserSensitive<'_>>) -> Result<MultiResponse, RocketCustom<String>> {
    let mut errors = Vec::new();
    handle_error!(post_usersensitive(&context, &sensitive).await, errors);
    Ok(MultiResponse::Template(userhome_base!(context, {sensitiveerrors:errors})))
}

#[post("/userhome", data= "<update>")]
pub async fn userhome_update_post(mut context: Context, update: Form<forms::UserUpdate<'_>>) -> Result<Template, RocketCustom<String>>
{
    let mut errors = Vec::new();
    //If the user is there, get a copy of it so we can modify and post it
    if let Some(mut current_user) = context.current_user.clone() {
        //Modify
        current_user.username = String::from(update.username);
        current_user.avatar = String::from(update.avatar);
        //Either update the context user or set an error
        match post_userupdate(&context, &current_user).await { 
            Ok(new_user) => context.current_user = Some(new_user),
            Err(error) => errors.push(error.to_string())
        }
    }
    else {
        errors.push(String::from("Couldn't pull user data, are you still logged in?"));
    }
    Ok(userhome_base!(context, {updateerrors:errors}))
}

//Don't need the heavy lifting of an entire context just for logout 
#[get("/logout")]
pub fn logout_get(config: &rocket::State<config::Config>, jar: &CookieJar<'_>) -> Redirect {
    jar.remove(Cookie::named(config.token_cookie_key.clone()));
    my_redirect!(config, "/")
}
