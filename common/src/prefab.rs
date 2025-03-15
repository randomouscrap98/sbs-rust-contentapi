// use crate::constants::*;
// use crate::response::*;
// use contentapi::conversion::*;
// use contentapi::endpoints::*;
// use contentapi::*;
// use serde_json::Value;

//This is for pre-constructed searches SPECIFICALLY within the API, hence "prefab".

// ----------------------
//    GENERAL CONTENT
// ----------------------

// #[derive(Default, Debug, Clone)]
// pub struct FullPage {
//     pub main: Content,
//     pub ptc: Option<Content>,
// }
//
// pub async fn get_fullpage(
//     context: &mut ApiContext,
//     by_field: &str,
//     value: Value,
// ) -> Result<FullPage, Error> {
//     let mut request = FullRequest::new();
//     let notfound = Error::NotFound(format!(
//         "Could not find content with {} = {}",
//         by_field, value
//     ));
//     add_value!(request, "findby", value);
//     add_value!(request, "ptcsystem", PTCSYSTEM);
//
//     let mut main_request = build_request!(
//         RequestType::content,
//         String::from("*"),
//         format!("{} = @findby", by_field)
//     );
//     main_request.limit = 1;
//     main_request.name = Some(String::from("main"));
//     request.requests.push(main_request);
//
//     let mut ptc_request = build_request!(
//         RequestType::content,
//         String::from("*"),
//         String::from("parentId = @main.id and literalType = @ptcsystem")
//     );
//     ptc_request.name = Some(String::from("ptc"));
//     request.requests.push(ptc_request);
//
//     let result = context.post_request(&request).await?;
//     let mut main = cast_result_required::<Content>(&result, "main")?;
//     let mut ptc = cast_result_required::<Content>(&result, "ptc")?;
//
//     Ok(FullPage {
//         main: main.pop().ok_or(notfound)?,
//         ptc: ptc.pop(),
//     })
// }

// pub async fn get_fullpage_by_hash(context: &mut ApiContext, hash: &str) -> Result<FullPage, Error> {
//     get_fullpage(context, "hash", hash.into()).await
// }

// pub async fn get_fullpage_by_id(context: &mut ApiContext, id: i64) -> Result<FullPage, Error> {
//     get_fullpage(context, "id", id.into()).await
// }
