use std::error::Error;

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

//pub fn cast_result<T>(result: &api_data::RequestResult, name: &str) -> Result<Option<Vec<T>>, Box<dyn Error>> where T: for<'a> Deserialize<'a>  
//Cast result; if key doesn't exist, you get none.
pub fn cast_result<T>(result: &api_data::RequestResult, name: &str) -> Result<Option<Vec<T>>, serde_json::Error> where T: for<'a> Deserialize<'a>  
{
    //If the key exists, do the conversion
    if let Some(content) = result.objects.get(name) {
        let mut items: Vec<T> = Vec::new();
        for c in content {
            items.push(<T as Deserialize>::deserialize(c)?);
        }
        Ok(Some(items))
    }
    else {
        Ok(None)
    }

}

//Cast result without care if the key exists. You'll get an empty vector
pub fn cast_result_safe<T>(result: &api_data::RequestResult, name: &str) -> Result<Vec<T>, serde_json::Error> where T: for<'a> Deserialize<'a>  
{
    cast_result(result, name).and_then(|r| Ok(r.unwrap_or(Vec::new())))
}

//Cast result but throw error if the key isn't found
pub fn cast_result_required<T>(result: &api_data::RequestResult, name: &str) -> Result<Vec<T>, Box<dyn Error>> where T: for<'a> Deserialize<'a>  
{
    cast_result(result, name)?.ok_or(format!("Couldn't find key {}", name).into())
}
