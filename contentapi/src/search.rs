use super::*;
use endpoints::*;
use conversion::*;
use crate as contentapi;

macro_rules! get_content_by_field {
    ($api:ident, $field:literal, $value:expr, $fields:expr) => {
        {
            //Why does this need another layer of brackets? Guess I'll have to look into it later
            let mut request = FullRequest::new();
            add_value!(request, "findby", $value);
            let mut creq = build_request!(
                RequestType::content,
                //Dont' need values for fpid, you already know it was there if it exists
                String::from($fields),
                format!("{} = @findby", $field)
            );
            creq.limit = 1; //Just in case
            request.requests.push(creq);

            let result = $api.post_request(&request).await?;
            let mut content = cast_result_required::<Content>(&result, "content")?;

            Ok(content.pop().ok_or_else(|| ApiError::Other(format!("Couldn't find content with {} {}", $field, $value)))?)
        }
    };
}

impl ApiContext 
{
    //These basic search things won't be profiled because we KNOW they're fast. If any are complex, 
    //we'll add an option to profile...
    pub async fn get_content_by_hash(&self, hash: &str, fields: &str) -> Result<Content, ApiError>
    {
        get_content_by_field!(self, "hash", hash, fields)
    }

    pub async fn get_content_by_id(&self, id: i64, fields: &str) -> Result<Content, ApiError>
    {
        get_content_by_field!(self, "id", id, fields)
    }
}