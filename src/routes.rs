
pub mod imagebrowser;
pub mod forums;
pub mod user;
pub mod register;
pub mod basic;

use crate::api::ApiError;
use rocket::http::Status;
use rocket::response;

#[derive(Debug, Responder)]
pub enum MultiResponse {
    Template(rocket_dyn_templates::Template),
    Redirect(rocket::response::Redirect),
}

//The over-arching error that I expect routes to emit
#[derive(Debug)]
pub struct RouteError(Status, String);

impl From<ApiError> for RouteError { 
    fn from(error: ApiError) -> Self {
        let status = match error {
            ApiError::Network(_) => Status::ServiceUnavailable,
            ApiError::User(_,_) => Status::BadRequest,
            _ => Status::InternalServerError
        };

        RouteError(status, error.to_string())
    }
}

impl From<anyhow::Error> for RouteError { 
    fn from(error: anyhow::Error) -> Self {
        RouteError(Status::InternalServerError, error.to_string())
    }
}

impl<'r, 'o: 'r> rocket::response::Responder<'r, 'o> for RouteError {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> response::Result<'o> {
        println!("[RETURNING:{}]: {}", &self.0, &self.1);
        let custom = rocket::response::status::Custom(self.0, self.1);
        custom.respond_to(req)
    }
}

macro_rules! my_redirect {
    ($config:expr, $location:expr) => {
        rocket::response::Redirect::to(format!("{}{}", $config.http_root, $location))
    };
}
pub(crate) use my_redirect;

//Most page rendering is exactly the same and requires the same base data.
//Just simplify that into a macro... (although we could probably do better)
macro_rules! basic_template{
    ($template:expr, $context:ident, {
        $($field_name:ident : $field_value:expr),*$(,)*
    }) => {
        rocket_dyn_templates::Template::render($template, rocket_dyn_templates::context! {
            //Only need to borrow everything from context, since it's all 
            //cloned values anyway. Also, this only works because context is passed
            //into the function as a guard, so the lifetime extends beyond the function
            //call and so can be part of the return value (being the template render)
            http_root : &$context.config.http_root,
            http_static : format!("{}/static", &$context.config.http_root),
            http_resources : format!("{}/static/resources", &$context.config.http_root),
            api_fileraw : &$context.config.api_fileraw, //These "api" endpoints from the configs are fullpaths? 
            //api_fileupload : &$context.config.api_fileupload, //so they can be moved
            route_path: &$context.route_path,
            route_uri: &$context.route_path,
            boot_time: &$context.init.boot_time,
            client_ip : &$context.client_ip,
            user: &$context.current_user,
            api_about: crate::api::get_about(&$context).await?,
            language_code: "en", //Eventually!!
            $($field_name: $field_value,)*
        })
    };
}
pub(crate) use basic_template;


macro_rules! login {
    ($jar:ident, $context: ident, $token: expr) => {
        login!($jar, $context, $token, 0) 
    };
    ($jar:ident, $context: ident, $token: expr, $expire: expr) => {
        //Again with the wasting memory and cpu, it's whatever. If we needed THAT much optimization,
        //uhh... well we'd have a lot of other problems than just a single small key copy on the heap
        let mut cookie = rocket::http::Cookie::build($context.config.token_cookie_key.clone(), $token);
        //Here, we say "if you send us an expiration, set the expiration. Otherwise, let it expire
        //at the end of the session"
        if $expire != 0 {
            cookie = cookie.max_age(rocket::time::Duration::seconds($expire as i64));
        }
        $jar.add(cookie.finish());
    };
}
pub(crate) use login;

//All our email endpoints in the API just return bools, handling them is a pain, lots of checking and building
macro_rules! handle_email {
    ($email_result:expr, $errors:ident) => {
        handle_error!($email_result, $errors, "Something went wrong sending the email! Try again?");
    };
}
pub(crate) use handle_email;

macro_rules! handle_error {
    ($result:expr, $errors:ident) => {
        handle_error!($result, $errors, "Unkown error occurred");
    };
    ($result:expr, $errors:ident, $message:literal) => {
        match $result //post_sendemail(context, email).await
        {
            //If confirmation is successful, we get a token back. We login and redirect to the userhome page
            Ok(success) => {
                if !success {
                    $errors.push(String::from($message));
                }
            },
            //If there's an error, we re-render the confirmation page with the errors.
            Err(error) => {
                $errors.push(error.to_string());
            } 
        }
    };
}
pub(crate) use handle_error;
