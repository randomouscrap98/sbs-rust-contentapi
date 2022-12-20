use contentapi::forms::UserSensitive;

use common::*;
use common::layout::*;
use maud::*;

pub fn render(data: MainLayoutData, errors: Option<Vec<String>>, email: Option<String>) -> String {
    layout(&data, html!{
        section {
            h1 {"Recover account"}
            p {"You'll receive an email shortly with the code to recover your account!"}
            form method="POST" action={(data.config.http_root)"/recover"} { //Must be exact!
                (errorlist(errors))
                label for="recover_email"{"Email (to identify account):"}
                input #"recover_email" type="email" required="" name="currentEmail" value=[&email];
                label for="recover_code" {"Recovery Code:"}
                input #"recover_code" type="text" required="" name="currentPassword";
                label for="recover_password"{"New password:"}
                input #"recover_password" type="password" required="" autocomplete="new-password" name="password";
                input type="submit" value="Recover account";
            }
            p{"Submitting this form will update the password for the account associated with the given email, and log you in."}
            @if email.is_none() {
                p."aside"
                {
                    r#"Did you get here by mistake? This page is meant to be used after sending a recovery email to your account.
                    It's still usable though, just make sure you get the code from your email! This page doesn't send emails!"#
                }
            }
        }
    }).into_string()
}


pub async fn post_render(context: PageContext, sensitive: &UserSensitive) -> (Response, Option<String>)
{
    match context.api_context.post_usersensitive(sensitive).await {
        Ok(token) => {
            (Response::Redirect(String::from("/userhome")), Some(token))
        },
        Err(error) => {
            (Response::Render(render(context.layout_data, Some(vec![error.to_user_string()]), Some(sensitive.currentEmail.clone()))), None)
        }
    }
}