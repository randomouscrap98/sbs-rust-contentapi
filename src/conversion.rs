use crate::context::Context;
use crate::forms;
use crate::api_data;

//Some data can't be used as-is. In those cases, we must translate from frontend formats to
//backend formats

//Produce the API login with the appropriate values. 
pub fn convert_login<'a>(context: &Context, login: &forms::Login<'a>) -> api_data::Login
{
    api_data::Login {
        username: String::from(login.username),
        password: String::from(login.password),
        expireSeconds : 
            if login.long_session { context.config.long_cookie_expire.into() }
            else { context.config.default_cookie_expire.into() }
    }
}
