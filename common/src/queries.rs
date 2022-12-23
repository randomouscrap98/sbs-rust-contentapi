use super::*;

use contentapi::*;
use contentapi::conversion::*;
use contentapi::endpoints::*;


pub async fn get_system_any(context: &mut ApiContext, ty: &str) -> Result<Option<Content>, Error> {
    let mut request = FullRequest::new();
    add_value!(request, "type", ContentType::SYSTEM);
    add_value!(request, "littype", ty);
    let alert_request = build_request!(
        RequestType::content,
        String::from("id,name,text,parentId,hash"),
        String::from("contentType = @type and literalType = @littype"),
        String::from("id") // Combined with 'pop', even if there are multiple alerts, we always get the last one
    );
    request.requests.push(alert_request);
    let result = context.post_request_profiled_opt(&request, "get-system").await?;
    let mut content = cast_result_required::<Content>(&result, "content")?;
    Ok(content.pop())
}

/// Returns the system alert; these should be in HTML format!
pub async fn get_system_alert(context: &mut ApiContext) -> Result<Option<Content>, Error> {
    get_system_any(context, &SBSContentType::alert.to_string()).await
}

/// Returns the frontpage; this shoudl be in HTML format!
pub async fn get_system_frontpage(context: &mut ApiContext) -> Result<Option<Content>, Error> {
    get_system_any(context, &SBSContentType::frontpage.to_string()).await
}