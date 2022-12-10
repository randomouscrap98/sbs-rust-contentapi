use super::*;
use contentapi;

use serde::{Serialize, Deserialize};

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
                    input #"login_extended" type="checkbox" name="long_session";
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
    pub long_session : bool,  //This is from the form itself, just a checkbox

    //While not really a great design IMO, this lets the caller pass values to us
    #[serde(skip)]
    pub long_session_seconds: i32,
    #[serde(skip)]
    pub default_session_seconds: i32
}

/// Produce the API login with the appropriate values. 
pub fn convert_login(login: Login) -> contentapi::forms::Login
{
    contentapi::forms::Login {
        username: login.username,
        password: login.password,
        expireSeconds : 
            if login.long_session { login.long_session_seconds.into() }
            else { login.default_session_seconds.into() }
    }
}

//pub async fn post_login_render(data: MainLayoutData, context: &contentapi::endpoints::ApiContext, login: Login) -> 
//    Result<Response, Error> 
//{
//    let api_login = convert_login(login);
//    let login_result = context.post_login(api_login).await?;
//}