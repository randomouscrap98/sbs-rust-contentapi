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
pub async fn login_get(context: Context) -> Result<Template, RouteError> {
    Ok(basic_template!("login", context, {}))
}

#[get("/userhome")]
pub async fn userhome_get(context: Context) -> Result<Template, RouteError> {
    Ok(userhome_base!(context, {}))
}

#[post("/login", data = "<login>")]
pub async fn login_post(context: Context, login: Form<forms::Login<'_>>, jar: &CookieJar<'_>) -> Result<MultiResponse, RouteError> {
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

//Alternate post endpoint for sending the recovery endpoint. On success, go to the /recover page, which
//will let you finalize setting a new password
#[post("/login?recover", data = "<recover>")]
pub async fn loginrecover_post(context: Context, recover: Form<forms::LoginRecover<'_>>) -> Result<MultiResponse, RouteError> {
    let mut errors = Vec::new();
    handle_email!(post_email_recover(&context, recover.email).await, errors);
    let template = if errors.len() == 0 { "recover" } else { "login" };
    //Error goes back to login template, but success goes to special reset page
    Ok(MultiResponse::Template(basic_template!(template, context, {
        emailresult : String::from(recover.email),  //This is 'email' because it's just SENDING the recovery email, not the recover form
        recovererrors: errors
    })))
}

#[get("/recover")] //A plain page render, if you accidentally get here. THe page will still work, but you have to add crap
pub async fn recover_get(context: Context) -> Result<Template, RouteError> {
    Ok(basic_template!("recover", context, { }))
}

//Dedicated recover submit page. On succcess, login and go to userhome. On failure, show recover page again
#[post("/recover", data = "<sensitive>")]
pub async fn recover_usersensitive_post(context: Context, sensitive: Form<forms::UserSensitive<'_>>, jar: &CookieJar<'_>) -> Result<MultiResponse, RouteError> {
    match post_usersensitive(&context, &sensitive).await {
        Ok(token) => {
            login!(jar, context, token);
            Ok(MultiResponse::Redirect(my_redirect!(context.config, "/userhome")))
        },
        Err(error) => {
            //This NEEDS to be the same as the post render from /login?recover!
            Ok(MultiResponse::Template(basic_template!("recover", context, {
                emailresult: String::from(sensitive.currentEmail),
                recovererrors: vec![error.to_string()]
            })))
        }
    }
}

//The userhome version of updating the sensitive info. This one actually has the ability to change your email
#[post("/userhome?sensitive", data = "<sensitive>")]
pub async fn usersensitive_post(context: Context, sensitive: Form<forms::UserSensitive<'_>>) -> Result<MultiResponse, RouteError> {
    let mut errors = Vec::new();
    match post_usersensitive(&context, &sensitive).await {
        Ok(_token) => {} //Don't need the token
        Err(error) => { errors.push(error.to_string()) }
    };
    Ok(MultiResponse::Template(userhome_base!(context, {sensitiveerrors:errors})))
}


#[post("/userhome", data= "<update>")]
pub async fn userhome_update_post(mut context: Context, update: Form<forms::UserUpdate<'_>>) -> Result<Template, RouteError>
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
