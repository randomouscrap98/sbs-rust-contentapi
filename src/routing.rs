use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router, extract::{DefaultBodyLimit, Query, FromRequestParts, Path}, async_trait, Form, http::StatusCode, response::IntoResponse, 
};

use tower_cookies::{CookieManagerLayer, Cookies, Cookie, cookie::{time::Duration, SameSite}};
use tower_http::{services::{ServeDir, ServeFile}, limit::RequestBodyLimitLayer};

use crate::state::{RequestContext, GlobalState};
use crate::srender;

pub mod login;
pub mod userhome;
pub mod admin;
pub mod user;
pub mod registerconfirm;
pub mod forum;

static SESSIONCOOKIE: &str = "sbs-rust-contentapi-session";
static SETTINGSCOOKIE: &str = "sbs-rust-contentapi-settings";

type StdResponse = Result<common::response::Response, common::response::Error>;

pub fn get_all_routes(gstate: Arc<GlobalState>) -> Router 
{
    //Only used for these routes!
    #[derive(serde::Deserialize, Debug)]
    struct SimplePage { page: Option<i32> }

    #[derive(serde::Deserialize, Default)]
    struct QrParam { high_density: Option<bool> }

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
        .route("/activity",
            get(|context: RequestContext, Query(search): Query<pages::activity::ActivityQuery>|
                srender!(pages::activity::get_render(context.page_context, search, context.global_state.config.default_activity_count))))
        .route("/search",
            get(|context: RequestContext, Query(search): Query<common::forms::PageSearch>|
                srender!(pages::search::get_render(context.page_context, search, context.global_state.config.default_display_pages))))
        .route("/allsearch", 
            get(|context: RequestContext, Query(search): Query<pages::searchall::SearchAllForm>| 
                srender!(pages::searchall::get_render(context.page_context, search))))
        .route("/login",
            get(|context: RequestContext| srender!(pages::login::get_render(context.page_context)))
            .post(login::login_post))
        .route("/userhome", 
            get(|context: RequestContext| srender!(pages::userhome::get_render(context.page_context)))
            .post(userhome::userhome_post))
        .route("/logout",
            get(|cookies: Cookies| async move {
                cookies.remove(Cookie::new(SESSIONCOOKIE, ""));
                common::response::Response::Redirect(String::from("/"))
            }))
        .route("/user/:username",
            get(|context: RequestContext, Path(username): Path<String>| 
                srender!(pages::user::get_render(context.page_context, username)))
            .post(user::user_post))
        .route("/admin", 
            get(|context: RequestContext, Query(search): Query<common::forms::AdminSearchParams>| 
                srender!(pages::admin::get_render(context.page_context, search)))
            .post(admin::admin_post))
        .route("/sessionsettings", 
            get(|context: RequestContext| 
                srender!(pages::sessionsettings::get_render(context.page_context)))
            .post(|mut context: RequestContext, cookies: Cookies, Form(form): Form<common::UserConfig>| async move {
                cookies.add(get_settings_cookie_convert(&form, &context.global_state.config)?);
                context.page_context.layout_data.user_config = form; //Is this safe? idk
                pages::sessionsettings::get_render(context.page_context).await
            }))
        .route("/register", 
            get(|context: RequestContext|  
                srender!(pages::register::get_render(context.page_context)))
            .post(|context: RequestContext, Form(form): Form<contentapi::forms::Register>|
                srender!(pages::register::post_render(context.page_context, &form))))
        .route("/register/confirm", 
            get(|context: RequestContext|  
                srender!(pages::registerconfirm::get_render(context.page_context)))
            .post(registerconfirm::registerconfirm_post))
        .route("/recover", 
            get(|context: RequestContext|  
                srender!(pages::recover::get_render(context.page_context)))
            .post(|context: RequestContext, cookies: Cookies, Form(form): Form<contentapi::forms::UserSensitive>| async move {
                let (response,token) = pages::recover::post_render(context.page_context, &form).await;
                if let Some(token) = token { cookies.add(get_new_login_cookie(token, context.global_state.config.default_cookie_expire as i64)); }
                StdResponse::Ok(response)
            }))
        .route("/forum", 
            get(forum::forum_get))
        .route("/forum/category/:hash", 
            get(|context: RequestContext, Path(hash): Path<String>, Query(page): Query<SimplePage>|
                srender!(pages::forum_category::get_hash_render(context.page_context, hash, context.global_state.config.default_display_threads, page.page))))
        .route("/forum/thread/:hash", 
            get(|context: RequestContext, Path(hash): Path<String>, Query(page): Query<SimplePage>|
                srender!(pages::forum_thread::get_hash_render(context.page_context, hash, context.global_state.config.default_display_posts, page.page))))
        .route("/forum/thread/:hash/:post", 
            get(|context: RequestContext, Path((hash,post)): Path<(String,i64)>|
                srender!(pages::forum_thread::get_hash_postid_render(context.page_context, hash, post, context.global_state.config.default_display_posts))))
        .route("/forum/delete/thread/:id",
            post(|context: RequestContext, Path(id): Path<i64>|
                srender!(pages::forum_edit_thread::delete_render(context.page_context, id))))
        .route("/forum/delete/post/:id",
            post(|context: RequestContext, Path(id): Path<i64>|
                srender!(pages::forum_edit_post::delete_render(context.page_context, id))))
        .route("/forum/edit/thread", 
            get(|context: RequestContext, Query(query): Query<forum::ThreadEditParameter>| 
                srender!(pages::forum_edit_thread::get_render(context.page_context, query.category, query.thread)))
            .post(|context: RequestContext, Form(form): Form<common::forms::ThreadForm>|
                srender!(pages::forum_edit_thread::post_render(context.page_context, form))))
        .route("/page/delete/:id",
            post(|context: RequestContext, Path(id): Path<i64>|
                srender!(pages::page_edit::delete_render(context.page_context, id))))
        .route("/page",
            get(|context: RequestContext, Query(query): Query<pages::page::PageQuery>|
                srender!(pages::page::get_pid_redirect(context.page_context, query))))
        .route("/widget/bbcodepreview", 
            get(|context: RequestContext| srender!(pages::widget_bbcodepreview::get_render(context.page_context)))
            .post(|context: RequestContext, Form(form) : Form<common::forms::BasicText>| 
                srender!(pages::widget_bbcodepreview::post_render(context.page_context, form.text))))
        .route("/widget/contentpreview", 
            post(|context: RequestContext, Form(form) : Form<pages::widget_contentpreview::ContentPreviewForm>| 
                srender!(pages::widget_contentpreview::post_render(context.page_context, form))))
        .route("/widget/imagebrowser", 
            get(|context: RequestContext, Query(query): Query<pages::widget_imagebrowser::Search>| 
                srender!(pages::widget_imagebrowser::query_render(context.page_context, query, context.global_state.config.default_imagebrowser_count))))
        .route("/widget/thread", 
            get(|context: RequestContext, Query(query): Query<common::forms::ThreadQuery>| 
                srender!(pages::widget_thread::get_render(context.page_context, query))))
        .route("/widget/votes/:id", 
            get(|context: RequestContext, Path(id): Path<i64>| 
                srender!(pages::widget_votes::get_render(context.page_context, id)))
            .post(|context: RequestContext, Path(id): Path<i64>, Form(form): Form<common::forms::VoteForm>|
                srender!(pages::widget_votes::post_render(context.page_context, id, form))))
        .route("/widget/recentactivity", 
            get(|context: RequestContext, Query(query): Query<pages::widget_recentactivity::RecentActivityConfig>| 
                srender!(pages::widget_recentactivity::get_render(context.page_context, query))))
        .route("/widget/qr/:hash", 
            get(|context: RequestContext, Path(hash): Path<String>, Query(query): Query<QrParam>| 
                srender!(pages::widget_qr::get_render(context.page_context, &hash, 
                    if let Some(hd) = query.high_density { hd } else { false }))))
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

fn get_new_settings_cookie(raw_settings: String, expire_seconds : i64) -> Cookie<'static> {
    Cookie::build(SETTINGSCOOKIE, raw_settings)
        .max_age(Duration::seconds(expire_seconds))
        .same_site(SameSite::Strict)
        .path("/")
        .finish()
}

fn get_settings_cookie_convert(form: &common::UserConfig, config: &crate::Config) -> Result<Cookie<'static>, common::response::Error>
{
    match serde_json::to_string(&form) {
        Ok(cookie) => Ok(get_new_settings_cookie(String::from(cookie), config.long_cookie_expire as i64)), //cookie_raw = Some(String::from(cookie)),
        Err(error) => Err(common::response::Error::Other(error.to_string()))
    }
}

//Produce an error response if a "typed" form does not include the type (those POST endpoints that
//accept multiple forms, and the type is the query parameter)
fn missing_type_response() -> axum::response::Response {
    (StatusCode::BAD_REQUEST, "Missing requisite submission type indicator (query parameter)").into_response()
}

#[macro_export]
macro_rules! srender {
    ($render:expr) => {
        async move {
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
            if let Some(query) = $req.uri().query() {
                let r = serde_urlencoded::from_str::<LocalQueryParam>(query);
                if r.is_ok() {
                    result = true;
                }
            }

            result
        }
    };
}

// Another silly thing for multi-route endpoints, parsing the form is always the same and we have to
// do it a million times.
#[macro_export]
macro_rules! parseform {
    ($wrapper:expr, $wrapped:ty, $req:expr) => {
        match Form::<$wrapped>::from_request($req, &()).await
        {
            Ok(Form(form)) => Ok($wrapper(form)),
            Err(e) => Err(e.into_response())
        }
    };
}

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
