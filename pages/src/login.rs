use super::*;
use common::*;
use common::forms::*;
use common::render::*;
use common::render::layout::*;
use common::response::*;
use contentapi;
use maud::*;

use serde::{Serialize, Deserialize};

pub fn render(data: MainLayoutData, login_errors: Option<Vec<String>>, recover_errors: Option<Vec<String>>, 
              email: Option<String>) -> String {
    layout(&data, html!{
        section {
            h1{"Login"}
            form method="POST" action={(data.links.http_root)"/login"} {
                (errorlist(login_errors))
                label for="login_username"{"Username:"}
                input #"login_username" type="text" required="" name="username";
                label for="login_password"{"Password:"}
                input #"login_password" type="password" required="" name="password";
                div."inline smallseparate" {
                    label for="login_extended" {"Very long session:"} 
                    input #"login_extended" type="checkbox" name="long_session" value="true";
                }
                input type="submit" value="Login";
            }
            hr;
            h2{"Password expired / forgotten?"}
            p.""{"Send an email with a temporary recovery code, which you can use to reset your password"}
            form method="POST" action={(data.links.http_root)"/login?recover=1"} {
                (errorlist(recover_errors))
                label for="recover_email" {"Email"}
                input #"recover_email" type="email" name="email" required="" value=[email];
                input type="submit" value="Send recovery email";
            }
            p."aside"{
                "Already have the recovery code? Go to the " 
                a href={(data.links.http_root)"/recover"} {"recovery page"}
                "."
            }
            hr;
            h2{"New to SmileBASIC Source?"}
            /* TODO: remove this when you're done! */
            //p."error" { "WARNING: ACCOUNT CREATION WILL GET RESET, THIS IS STILL A TEST WEBSITE!" }
            p { a href={(data.links.http_root)"/register"}{"Register here"} }
            p."aside" { 
                "If you already registered and need to enter the confirmation code, go to the " 
                a href={(data.links.http_root)"/register/confirm"}{ "confirmation page" }
                "."
            }
        }
    }).into_string()
}

pub async fn get_render(context: PageContext) -> Result<Response, Error> {
    Ok(Response::Render(render(context.layout_data, None, None, None)))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Login
{
    pub username: String,
    pub password: String,
    pub long_session : Option<bool>,  //This is from the form itself, just a checkbox
}

impl Login {
    /// Produce the API login with the appropriate values. 
    pub fn to_api_login(self, default_seconds: i32, long_seconds: i32) -> contentapi::forms::Login
    {
        let long_session = self.long_session.unwrap_or(false);
        contentapi::forms::Login {
            username: self.username,
            password: self.password,
            expireSeconds : 
                if long_session { long_seconds.into() }
                else { default_seconds.into() }
        }
    }
}


/// Rendering for posting a user login. But, may redirect instead! You have to inspect the Response! On success,
/// the Ok result has a string as well, that's the token. There's no way for this to fail, as one way or another,
/// you're going to get a response
pub async fn post_login_render(context: PageContext, login: &contentapi::forms::Login) -> 
    (Response, Option<String>)
{
    match context.api_context.post_login(login).await {
        Ok(token) => (Response::Redirect(String::from("/userhome")), Some(token)),
        Err(error) => {
            println!("Login raw error: {}", error.to_verbose_string());
            (Response::Render(render(context.layout_data, Some(vec![error.to_user_string()]), None, None)), None)
        }
    }
}

/// Account recovery just requires an email, which we will try to send the recovery code to. On 
/// success, we render the recovery page. Otherwise, we render the login page again (all on same url)
pub async fn post_login_recover(context: PageContext, recover: &EmailGeneric) -> Response
{
    let email = recover.email.clone(); //make a copy for later
    let errors = email_errors!(context.api_context.post_email_recover(&recover.email).await);
    if errors.len() == 0 { //Success!
        Response::Render(recover::render(context.layout_data, None, Some(email)))
    }
    else { //Failure! Re-render page with errors
        Response::Render(render(context.layout_data, None, Some(errors), Some(email)))
    }
}