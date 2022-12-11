use contentapi::{User, endpoints::ApiContext};

use super::*;

pub fn render(data: MainLayoutData, confirm_errors: Option<Vec<String>>, email_errors: Option<Vec<String>>,
    email: Option<String>, user: Option<User>, resend_success: bool) -> String 
{
    layout(&data, html!{
        section {
            h1 { "Complete Registration" }
            @if let Some(_email) = &email {
                @if let Some(user) = &user {
                    p { "Hello, "(user.username)"!"}
                }
                p { 
                    "You'll receive an email shortly from smilebasicsource@gmail.com with the to code to complete your "
                    "registration, enter it below. If you " span."error"{"reload or navigate away from the page"}", "
                    "you can still complete your registration, you'll just have to supply your email."
                }
            }
            @else {
                p {"If you've already registered, you'll receive a confirmation email shortly. Re-enter your email and the "
                   "confirmation code here to complete your registration." }
            }
            form method="POST" action={(data.config.http_root)"/register/confirm"} {
                (errorlist(confirm_errors))
                label for="complete_email" {"Email:"}
                input #"complete_email" type="text" name="email" required="" value=[&email];
                label for="complete_key" {"Code from email:"}
                input."largeinput" #"complete_key" required="" type="text" name="key";
                input type="submit" value="Complete registration";
            }
            hr;
            h3 {"Didn't get the email?"}
            p {"The email comes from smilebasicsource@gmail.com. It may be in your spam folder, and it may take up to a couple minutes "
               "to get through email filters. If you didn't receive it, you can send it again here:" }
            //Post to the special endpoint still under the "confirm" umbrella, so errors will be rendered "on the same page"
            form method="POST" action={(data.config.http_root)"/register/confirm?resend=1"} {
                (errorlist(email_errors))
                @if resend_success {
                    p."success"{"Email resent!"}
                }
                label for="resend_email" {"Email:"}
                input #"resend_email" type="text" name="email" required="" value=[&email];
                input type="submit" value="Resend confirmation email";
            }
        }
    }).into_string()
}

/// Regular confirmation acceptance. On success, we redirect you to your userhome while returning the token (presumably
/// to log you in on whatever routing you have). On failure, we re-render the confirmation page with the errors.
pub async fn post_render(data: MainLayoutData, context: &ApiContext, confirm: &contentapi::forms::RegisterConfirm) -> 
    (Response, Option<String>)
{
    let email = confirm.email.clone(); //For use later
    match context.post_register_confirm(confirm).await
    {
        //If confirmation is successful, we get a token back. We login and redirect to the userhome page
        Ok(token) => {
            (Response::Redirect(String::from("/userhome")), Some(token))
        },
        //If there's an error, we re-render the confirmation page with the errors.
        Err(error) => {
            (Response::Render(render(data, Some(vec![error.to_user_string()]), None, Some(email), None, false)), None)
        }
    }
}

/// Resend the confirmation email. On both success and failure, it re-renders the page, just with different elements
/// in the resend form for success or failure.
pub async fn post_email_render(data: MainLayoutData, context: &ApiContext, resend: &EmailGeneric) -> Response 
{
    let email = resend.email.clone(); //make a copy for later
    let errors = email_errors!(context.post_email_sendregistration(&email).await);
    if errors.len() == 0 { 
        //Success! Rerender the current page with success set (no errors)
        Response::Render(render(data, None, None, Some(email), None, true))
    }
    else { 
        //Failure! Re-render page with errors
        Response::Render(render(data, None, Some(errors), Some(email), None, false))
    }
}