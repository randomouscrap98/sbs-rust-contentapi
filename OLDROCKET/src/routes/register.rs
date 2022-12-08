use crate::context::*;
use crate::forms;
use crate::api::*;
use super::*;

use rocket::http::{CookieJar};
use rocket::form::Form;
use rocket_dyn_templates::Template;

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

#[get("/register")] 
pub async fn register_get(context: Context) -> Result<Template, RouteError> {
    Ok(register_base!(context, {}))
}

#[get("/register/confirm")] //This is a PLAIN confirmation page with no extra data
pub async fn registerconfirm_get(context: Context) -> Result<Template, RouteError> {
    Ok(registerconfirm_base!(context, { }))
}

#[post("/register", data = "<registration>")]
pub async fn register_post(context: Context, registration: Form<forms::Register<'_>>) -> Result<MultiResponse, RouteError> {
    match post_register(&context, &registration).await
    {
        //On success, we render the confirmation page with the email result baked in (it's more janky because it's
        //the same page data but on the same route but whatever... it's safer).
        Ok(userresult) => {
            let mut errors = Vec::new();
            //Oh but if the email fails, we need to tell them about it. 
            handle_email!(post_email_confirm(&context, registration.email).await, errors);
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
            Ok(MultiResponse::Template(register_base!(context, {errors: vec![error.to_string()]})))
        } 
    }
}

#[post("/register/confirm", data = "<confirm>")]
pub async fn registerconfirm_post(context: Context, confirm: Form<forms::RegisterConfirm<'_>>, jar: &CookieJar<'_>) -> Result<MultiResponse, RouteError> {
    match post_registerconfirm(&context, &confirm).await
    {
        //If confirmation is successful, we get a token back. We login and redirect to the userhome page
        Ok(token) => {
            //Registration provides no expiration, so we let the cookie expire as soon as possible
            login!(jar, context, token);
            Ok(MultiResponse::Redirect(my_redirect!(context.config, format!("/userhome"))))
        },
        //If there's an error, we re-render the confirmation page with the errors.
        Err(error) => {
            Ok(MultiResponse::Template(registerconfirm_base!(context, {errors: vec![error.to_string()]})))
        } 
    }
}

#[post("/register/confirm?resend", data = "<resendform>")]
pub async fn registerresend_post(context: Context, resendform: Form<forms::RegisterResend<'_>>) -> Result<MultiResponse, RouteError> {
    let mut errors = Vec::new();
    handle_email!(post_email_confirm(&context, resendform.email).await, errors);
    Ok(MultiResponse::Template(registerconfirm_base!(context, {
        emailresult : String::from(resendform.email), 
        resendsuccess: errors.len() == 0, 
        resenderrors: errors
    })))
}
