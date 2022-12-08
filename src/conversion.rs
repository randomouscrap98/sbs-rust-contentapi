//use std::error::Error;

use std::fmt::Display;

use serde::Deserialize;

//use crate::api::Context; //context::Context;
//use crate::api::forms;
use crate::api;
//use anyhow::anyhow;

//Some data can't be used as-is. In those cases, we must translate from frontend formats to
//backend formats

////Produce the API login with the appropriate values. 
//pub fn convert_login<'a>(context: &Context, login: &forms::Login<'a>) -> api_data::Login
//{
//    api_data::Login {
//        username: String::from(login.username),
//        password: String::from(login.password),
//        expireSeconds : 
//            if login.long_session { context.config.long_cookie_expire.into() }
//            else { context.config.default_cookie_expire.into() }
//    }
//}

//pub fn cast_result<T>(result: &api_data::RequestResult, name: &str) -> Result<Option<Vec<T>>, Box<dyn Error>> where T: for<'a> Deserialize<'a>  
//Cast result; if key doesn't exist, you get none.
pub fn cast_result<T>(result: &api::RequestResult, name: &str) -> Result<Option<Vec<T>>, Box<dyn std::error::Error>> where T: for<'a> Deserialize<'a>  
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
pub fn cast_result_safe<T>(result: &api::RequestResult, name: &str) -> Result<Vec<T>, Box<dyn std::error::Error>> where T: for<'a> Deserialize<'a>  
{
    cast_result(result, name).and_then(|r| Ok(r.unwrap_or(Vec::new())))
}

//Cast result but throw error if the key isn't found
pub fn cast_result_required<T>(result: &api::RequestResult, name: &str) -> Result<Vec<T>, Box<dyn std::error::Error>> where T: for<'a> Deserialize<'a>  
{
    cast_result(result, name)?.ok_or(format!("Couldn't find key {}", name).into())
}

//This gets rid of our dependency on serde_qs
//pub fn list_to_querystring(list: Vec<(String, Option<impl Display>)>) -> String {
//    if list.len() == 0 {
//        String::new()
//    }
//    else {
//        format!("?{}", list.iter()
//            .map(|(key,value)| urlencoding::encode(&i.to_string()).into_owned())
//            .collect::<Vec<String>>()
//            .join("&")
//        )
//    }
//}
//
//#[cfg(test)]
//mod tests {
//    use super::*;
//
//    macro_rules! querystring_tests {
//        ($($name:ident: $value:expr;)*) => {
//        $(
//            #[test]
//            fn $name() {
//                let (input, expected) = $value;
//                assert_eq!(list_to_querystring(input), String::from(expected));
//            }
//        )*
//        }
//    }
//
//    querystring_tests! {
//        empty: (Vec::<String>::new(), "");
//        single: (vec![("wow", None)], "");
//    }
//}