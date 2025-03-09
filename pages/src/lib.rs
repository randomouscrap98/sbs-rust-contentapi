pub mod about;
pub mod activity;
pub mod admin;
pub mod documentation;
pub mod forum_category;
pub mod forum_edit_post;
pub mod forum_edit_thread;
pub mod forum_main;
pub mod forum_thread;
pub mod index;
pub mod integrationtest;
pub mod login;
pub mod page;
pub mod recover;
pub mod search;
pub mod searchall;
pub mod sessionsettings;
pub mod user;
pub mod widget_qr;
pub mod widget_recentactivity;
pub mod widget_thread;
pub mod widget_votes;

//Email errors are weird with their true/false return.
macro_rules! email_errors {
    ($result:expr) => {
        {
            let mut errors: Vec<String> = Vec::new();
            match $result //post_sendemail(context, email).await
            {
                //If confirmation is successful, we get a token back. We login and redirect to the userhome page
                Ok(success) => {
                    if !success {
                        errors.push(String::from("Unkown error (email endpoint returned false!)"));
                    }
                },
                //If there's an error, we re-render the confirmation page with the errors.
                Err(error) => {
                    println!("Email endpoint raw error: {}", error.to_verbose_string());
                    errors.push(error.to_user_string());
                }
            }
            errors
        }
    };
}
pub(crate) use email_errors;
