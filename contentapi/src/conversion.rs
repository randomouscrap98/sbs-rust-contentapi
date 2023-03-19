use super::*;

use serde::Deserialize;

//Some data can't be used as-is. In those cases, we must translate from frontend formats to
//backend formats

/// Cast result; if key doesn't exist, you get none.
pub fn cast_result<T>(result: &RequestResult, name: &str) -> Result<Option<Vec<T>>, Box<dyn std::error::Error>> where T: for<'a> Deserialize<'a>  
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

/// Cast result without care if the key exists. You'll get an empty vector
pub fn cast_result_safe<T>(result: &RequestResult, name: &str) -> Result<Vec<T>, Box<dyn std::error::Error>> where T: for<'a> Deserialize<'a>  
{
    cast_result(result, name).and_then(|r| Ok(r.unwrap_or(Vec::new())))
}

/// Cast result but throw error if the key isn't found
pub fn cast_result_required<T>(result: &RequestResult, name: &str) -> Result<Vec<T>, Box<dyn std::error::Error>> where T: for<'a> Deserialize<'a>  
{
    cast_result(result, name)?.ok_or(format!("Couldn't find key {}", name).into())
}
