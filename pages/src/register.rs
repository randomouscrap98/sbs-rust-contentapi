
use super::*;
use common::*;
use common::render::*;
use common::render::layout::*;
use maud::*;

pub fn render(data: MainLayoutData, errors: Option<Vec<String>>, username: Option<String>, email: Option<String>) -> String {
    layout(&data, html!{
        section {
            h1 { "Register" }
            form method="POST" action=(data.links.http_root) {
                (errorlist(errors))
                label for="register_username" {"Username:"}
                input #"register_username" type="text" name="username" value=[username];
                label for="register_password" {"Password:"}
                input #"register_password" type="password" name="password";
                label for="register_email" {"Email:"}
                input #"register_email" type="email" name="email" value=[email];
                p."aside" { 
                    "We only use your email for account recovery and verification. All code is open source, see: "
                    a href={(data.links.http_root)"/about"} {"About"}
                }
                input type="submit" value="Register";
            }
        }
    }).into_string()
}



pub async fn post_render(context: PageContext, registration: &contentapi::forms::Register) -> Response 
{
    let email = registration.email.clone(); //make a copy for later
    let username = registration.username.clone();
    match context.api_context.post_register(registration).await //the initial registration
    {
        //On success, we render the confirmation page with the email result baked in (it's more janky because it's
        //the same page data but on the same route but whatever... it's safer).
        Ok(userresult) => {
            //Gotta send out the registration email though, since it's a two step process in the API
            let errors = email_errors!(context.api_context.post_email_sendregistration(&email).await);
            if errors.len() == 0 { 
                //On success, we show the user the confirmation page with their information
                Response::Render(registerconfirm::render(context.layout_data, None, None, Some(email), Some(userresult), false))
            }
            else {
                //Oh but if the email fails, we need to tell them about it. 
                Response::Render(render(context.layout_data, Some(errors), Some(username), Some(email)))
            }
        },
        Err(error) => {
            //On failure, we re-render the registration page, show errors
            Response::Render(render(context.layout_data, Some(vec![error.to_user_string()]), Some(username), Some(email)))
        } 
    }
}