use serde::Deserialize;

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

pub fn cast_result<T>(result: &api_data::RequestResult, name: &str) -> Result<Vec<T>, serde_json::Error> where T: for<'a> Deserialize<'a>  {
    let mut items: Vec<T> = Vec::new();

    if let Some(content) = result.objects.get(name) {
        for c in content {
            items.push(<T as Deserialize>::deserialize(c)?);
        }
    }

    Ok(items)
}