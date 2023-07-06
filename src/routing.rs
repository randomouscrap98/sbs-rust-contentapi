use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router, extract::{DefaultBodyLimit, Query, FromRequestParts}, async_trait, Form, 
};

//use serde::Deserialize;
use tower_cookies::{CookieManagerLayer, Cookies, Cookie, cookie::{time::Duration, SameSite}};
use tower_http::{services::{ServeDir, ServeFile}, limit::RequestBodyLimitLayer};

use crate::state::{RequestContext, GlobalState};
use crate::srender;

pub mod login;

static SESSIONCOOKIE: &str = "sbs-rust-contentapi-session";
static SETTINGSCOOKIE: &str = "sbs-rust-contentapi-settings";

type StdResponse = Result<common::response::Response, common::response::Error>;

pub fn get_all_routes(gstate: Arc<GlobalState>) -> Router 
{
    // build our application with a route
    let app = Router::new()
        .route("/", 
            get(|context: RequestContext| srender!(pages::index::get_render(context.page_context))))
        .route("/about", 
            get(|context: RequestContext| srender!(pages::about::get_render(context.page_context))))
        .route("/integrationtest", 
            get(|context: RequestContext| srender!(pages::integrationtest::get_render(context.page_context))))
        .route("/documentation", 
            get(|context: RequestContext| srender!(pages::documentation::get_render(context.page_context))))
        .route("/allsearch", 
            get(|context: RequestContext, Query(search): Query<pages::searchall::SearchAllForm>| 
                srender!(pages::searchall::get_render(context.page_context, search))))
        .route("/login",
            get(|context: RequestContext| srender!(pages::login::get_render(context.page_context)))
            .post(login::login_post)
        )
        .route("/logout",
            get(|cookies: Cookies| async move {
                cookies.remove(Cookie::new(SESSIONCOOKIE, ""));
                common::response::Response::Redirect(String::from("/"))
            }))
        .route("/widget/bbcodepreview", 
            get(|context: RequestContext| srender!(pages::widget_bbcodepreview::get_render(context.page_context)))
            .post(|context: RequestContext, Form(form) : Form<common::forms::BasicText>| 
                srender!(pages::widget_bbcodepreview::post_render(context.page_context, form.text))))
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/favicon.ico", ServeFile::new("static/resources/favicon.ico"))
        .nest_service("/robots.txt", ServeFile::new("static/robots.txt"))
        .with_state(gstate.clone())
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(
            gstate.config.body_maxsize as usize
        ))
        .layer(CookieManagerLayer::new())
    ;

    app
}

//Generate a new login cookie with all the bits and bobs set appropriately
fn get_new_login_cookie(token: String, expire_seconds : i64) -> Cookie<'static> {
    Cookie::build(SESSIONCOOKIE, token)
        .max_age(Duration::seconds(expire_seconds))
        .same_site(SameSite::Strict)
        .path("/")
        .finish()
}

#[macro_export]
macro_rules! srender {
    ($render:expr) => {
        async {
            StdResponse::Ok($render.await?)
        }
    };
}

/// Silly thing to limit a route by a single flag present (must be i8)
#[macro_export]
macro_rules! qflag {
    ($flag:ident, $req:expr) => {
        {
            #[allow(dead_code)]
            #[derive(serde::Deserialize)]
            struct LocalQueryParam { $flag: i8 }

            let mut result = false;
            let uri = $req.uri();
            let query = uri.query();
            if let Some(query) = query {
                let r = serde_urlencoded::from_str::<LocalQueryParam>(query);
                if r.is_ok() {
                    result = true;
                }
            }

            result
            ////Hopefully this doesn't fully consume the request?
            ////let parts = req.extract_parts();
            //Query::<LocalQueryParam>::from_request($req, $state)
            //    .await.is_ok()
        }
    };
}


/*
    Issues:
    ---------
    * Pages seem to return both a response and an error, when the response usually indicates the entire error? Could maybe flatten
      into just "response" and make a way to produce the appropriate output? Are there any routes that might have errors before
      you get to the rendering? Maybe for the special routes that are either/or... let's assume Response is always enough though.
    * 
 */


#[async_trait]
impl FromRequestParts<Arc<GlobalState>> for RequestContext
{
    type Rejection = common::response::Error;

    async fn from_request_parts(parts: &mut axum::http::request::Parts, state: &Arc<GlobalState>) -> Result<Self, Self::Rejection>
    {
        use axum::RequestPartsExt;
        let cookies = parts.extract::<Cookies>()
            .await
            .map_err(|err| Self::Rejection::Other(err.1.to_string()))?;
        let path = parts.extract::<axum::http::Uri>()
            .await.unwrap(); //Infallible?

        let token = cookies.get(SESSIONCOOKIE).and_then(|t| Some(t.value().to_string()));
        let config_raw = cookies.get(SETTINGSCOOKIE).and_then(|c| Some(c.value().to_string()));
        RequestContext::generate(state.clone(), &path.to_string(), token, config_raw).await
    }
}
