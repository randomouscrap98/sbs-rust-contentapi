use super::*;
use contentapi;

use serde::{Serialize, Deserialize};
use serde_aux::prelude::deserialize_bool_from_anything;

pub fn render(data: MainLayoutData, login_errors: Option<Vec<String>>, recover_errors: Option<Vec<String>>, 
              email: Option<String>) -> String {
    layout(&data, html!{
        section {
            h1{"Login"}
            form method="POST" action={(data.config.http_root)"/login"} {
                (errorlist(login_errors))
                label for="login_username"{"Username:"}
                input #"login_username" type="text" required="" name="username";
                label for="login_password"{"Password:"}
                input #"login_password" type="password" required="" name="password";
                label."inline" for="login_extended"{
                    span{"Very long session:"} 
                    input #"login_extended" type="checkbox" name="long_session" value="true";
                }
                input type="submit" value="Login";
            }
            hr;
            h2{"Password expired / forgotten?"}
            p.""{"Send an email with a temporary recovery code, which you can use to reset your password"}
            form method="POST" action={(data.config.http_root)"/login?recover"} {
                (errorlist(recover_errors))
                label for="recover_email"{"Email"}
                input #"recover_email" type="text" name="email" required="" value=[email];
                input type="submit" value="Send recovery email";
            }
            p."aside"{
                "Already have the recovery code? Go to the " 
                a href={(data.config.http_root)"/recover"} {"recovery page"}
                "."
            }
            hr;
            h2{"New to SmileBASIC Source?"}
            p { a href={(data.config.http_root)"/register"}{"Click here"} " to register" }
        }
    }).into_string()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Login
{
    pub username: String,
    pub password: String,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub long_session : bool,  //This is from the form itself, just a checkbox

    ////While not really a great design IMO, this lets the caller pass values to us
    //#[serde(skip)]
    //pub long_session_seconds: i32,
    //#[serde(skip)]
    //pub default_session_seconds: i32
}

impl Login {
    /// Produce the API login with the appropriate values. 
    pub fn to_api_login(self, default_seconds: i32, long_seconds: i32) -> contentapi::forms::Login
    {
        contentapi::forms::Login {
            username: self.username,
            password: self.password,
            expireSeconds : 
                if self.long_session { long_seconds.into() }
                else { default_seconds.into() }
        }
    }
}

/// Rendering for posting a user login. But, may redirect instead! You have to inspect the Response! On success,
/// the Ok result has a string as well, that's the token. There's no way for this to fail, as one way or another,
/// you're going to get a response
pub async fn post_login_render(data: MainLayoutData, context: &contentapi::endpoints::ApiContext, login: &contentapi::forms::Login) -> 
    (Response, Option<String>)
{
    match context.post_login(login).await {
        Ok(token) => (Response::Redirect(String::from("/userhome")), Some(token)),
        Err(error) => {
            println!("Login raw error: {}", error.to_verbose_string());
            (Response::Render(render(data, Some(vec![error.to_user_string()]), None, None)), None)
        }
    }
}

pub async fn post_login_recover(data: MainLayoutData, context: &contentapi::endpoints::ApiContext, 
    recover: &contentapi::forms::EmailGeneric) -> Response
{
    let email = recover.email.clone(); //make a copy for later
    match context.post_email_recover(recover).await {
        Ok(success) => {
            if success {
                //Render the recover page with this email!
                Response::Render(recover::render(data, None, Some(email)))
            }
            else {
                Response::Render(render(data, None, Some(vec![String::from("Unknown error (email endpoint returned false!)")]), Some(email)))
            }
        },
        Err(error) => {
            println!("Recover raw error: {}", error.to_verbose_string());
            Response::Render(render(data, None, Some(vec![error.to_user_string()]), Some(email)))
        }
    }
}