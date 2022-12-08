
//Need the basic "get" route, which should return a template html

#[get("/")]
pub async fn index_get(context: Context) -> Result<Template, RouteError> {
    Ok(basic_template!("index", context, {}))
}