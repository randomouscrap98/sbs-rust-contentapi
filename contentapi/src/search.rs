use super::*;
use endpoints::*;
use conversion::*;
use crate as contentapi;

macro_rules! get_something_by_field {
    ($api:ident, $reqtype:expr, $cast_type:ty, $field:literal, $value:expr, $fields:expr) => {
        {
            //Why does this need another layer of brackets? Guess I'll have to look into it later
            let mut request = FullRequest::new();
            add_value!(request, "findby", $value);
            let mut creq = build_request!(
                $reqtype,
                //Dont' need values for fpid, you already know it was there if it exists
                String::from($fields),
                format!("{} = @findby", $field)
            );
            creq.limit = 1; //Just in case
            request.requests.push(creq);

            let result = $api.post_request(&request).await?;
            let mut content = cast_result_required::<$cast_type>(&result, &$reqtype.to_string())?;

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
        get_something_by_field!(self, RequestType::content, Content, "hash", hash, fields)
    }

    pub async fn get_content_by_id(&self, id: i64, fields: &str) -> Result<Content, ApiError>
    {
        get_something_by_field!(self, RequestType::content, Content, "id", id, fields)
    }

    pub async fn get_message_by_id(&self, id: i64, fields: &str) -> Result<Message, ApiError>
    {
        get_something_by_field!(self, RequestType::message, Message, "id", id, fields)
    }
}