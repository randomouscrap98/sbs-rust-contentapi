use contentapi::User;

use super::*;

pub fn render(data: MainLayoutData, register_errors: Option<Vec<String>>, email_errors: Option<Vec<String>>,
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
                (errorlist(register_errors))
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
