use std::sync::Arc;

use axum::{
    async_trait,
    extract::{DefaultBodyLimit, FromRequestParts, Path, Query},
    routing::get,
    Form, Router,
};

use tower_cookies::{
    cookie::{time::Duration, SameSite},
    Cookie, CookieManagerLayer, Cookies,
};
use tower_http::{
    limit::RequestBodyLimitLayer,
    services::{ServeDir, ServeFile},
};

use crate::layout::*;
use crate::state::*;
//use crate::pages::context::*;
use crate::response::*;
use crate::srender; //state::{GlobalState, RequestContext};

static SETTINGSCOOKIE: &str = "sbs-rust-contentapi-settings";

type StdResponse = Result<Response, Error>;

pub fn get_all_routes(gstate: Arc<GlobalState>) -> Router {
    //Only used for these routes!
    #[derive(serde::Deserialize, Debug)]
    struct SimplePage {
        page: Option<i32>,
    }

    #[derive(serde::Deserialize, Default)]
    struct QrParam {
        high_density: Option<bool>,
    }

    // build our application with a route
    let app =
        Router::new()
            .route(
                "/",
                get(|context: RequestContext| {
                    srender!(crate::pages::index::get_render(context.page_context))
                }),
            )
            .route(
                "/about",
                get(|context: RequestContext| {
                    srender!(crate::pages::about::get_render(context.page_context))
                }),
            )
            .route(
                "/documentation",
                get(|context: RequestContext| {
                    srender!(crate::pages::documentation::get_render(
                        context.page_context
                    ))
                }),
            )
            .route(
                "/search",
                get(
                    |context: RequestContext,
                     Query(search): Query<crate::pages::common::PageSearch>| {
                        srender!(crate::pages::search::get_render(
                            context.page_context,
                            search,
                            context.global_state.config.default_display_pages
                        ))
                    },
                ),
            )
            .route(
                "/allsearch",
                get(
                    |context: RequestContext, Query(search): Query<crate::pages::searchall::SearchAllForm>| {
                        srender!(crate::pages::searchall::get_render(context.page_context, search))
                    },
                ),
            )
            .route(
                "/user/:username",
                get(|context: RequestContext, Path(username): Path<String>| {
                    srender!(crate::pages::user::get_render(context.page_context, username))
                }),
            )
            .route(
                "/sessionsettings",
                get(|context: RequestContext| {
                    srender!(crate::pages::sessionsettings::get_render(
                        context.page_context
                    ))
                })
                .post(
                    |mut context: RequestContext,
                     cookies: Cookies,
                     Form(form): Form<UserConfig>| async move {
                        cookies.add(get_settings_cookie_convert(
                            &form,
                            &context.global_state.config,
                        )?);
                        context.page_context.layout_data.user_config = form; //Is this safe? idk
                        crate::pages::sessionsettings::get_render(context.page_context).await
                    },
                ),
            )
            .route("/forum", get(forum_get))
            .route(
                "/forum/category/:hash",
                get(
                    |context: RequestContext,
                     Path(hash): Path<String>,
                     Query(page): Query<SimplePage>| {
                        srender!(crate::pages::forum_category::get_hash_render(
                            context.page_context,
                            hash,
                            context.global_state.config.default_display_threads,
                            page.page
                        ))
                    },
                ),
            )
            .route(
                "/forum/thread/:hash",
                get(
                    |context: RequestContext,
                     Path(hash): Path<String>,
                     Query(page): Query<SimplePage>| {
                        srender!(crate::pages::forum_thread::get_hash_render(
                            context.page_context,
                            hash,
                            context.global_state.config.default_display_posts,
                            page.page
                        ))
                    },
                ),
            )
            .route(
                "/forum/thread/:hash/:post",
                get(
                    |context: RequestContext, Path((hash, post)): Path<(String, i64)>| {
                        srender!(crate::pages::forum_thread::get_hash_postid_render(
                            context.page_context,
                            hash,
                            post,
                            context.global_state.config.default_display_posts
                        ))
                    },
                ),
            )
            .route(
                "/page",
                get(
                    |context: RequestContext,
                     Query(query): Query<crate::pages::redirect::PageQuery>| {
                        srender!(crate::pages::redirect::get_pid_redirect(
                            context.page_context,
                            query
                        ))
                    },
                ),
            )
            // .route(
            //     "/widget/thread",
            //     get(
            //         |context: RequestContext, Query(query): Query<common::forms::ThreadQuery>| {
            //             srender!(pages::widget_thread::get_render(
            //                 context.page_context,
            //                 query
            //             ))
            //         },
            //     ),
            // )
            .route(
                "/widget/votes/:id",
                get(|context: RequestContext, Path(id): Path<i64>| {
                    srender!(crate::pages::widget_votes::get_render(context.page_context, id))
                }),
            )
            .route(
                "/widget/qr/:hash",
                get(
                    |context: RequestContext,
                     Path(hash): Path<String>,
                     Query(query): Query<QrParam>| {
                        srender!(crate::pages::widget_qr::get_render(
                            context.page_context,
                            &hash,
                            if let Some(hd) = query.high_density {
                                hd
                            } else {
                                false
                            }
                        ))
                    },
                ),
            )
            .nest_service("/static", ServeDir::new("static"))
            .nest_service("/uploads/files", ServeDir::new(&gstate.config.upload_dir))
            .nest_service(
                "/uploads/thumbnails",
                ServeDir::new(&gstate.config.thumbnail_dir),
            )
            .nest_service(
                "/favicon.ico",
                ServeFile::new("static/resources/favicon.ico"),
            )
            .nest_service("/robots.txt", ServeFile::new("static/robots.txt"))
            .with_state(gstate.clone())
            .layer(DefaultBodyLimit::disable())
            .layer(RequestBodyLimitLayer::new(
                gstate.config.body_maxsize as usize,
            ))
            .layer(CookieManagerLayer::new());

    app
}

fn get_new_settings_cookie(raw_settings: String, expire_seconds: i64) -> Cookie<'static> {
    Cookie::build(SETTINGSCOOKIE, raw_settings)
        .max_age(Duration::seconds(expire_seconds))
        .same_site(SameSite::Strict)
        .path("/")
        .finish()
}

fn get_settings_cookie_convert(
    form: &UserConfig,
    config: &crate::Config,
) -> Result<Cookie<'static>, Error> {
    match serde_json::to_string(&form) {
        Ok(cookie) => Ok(get_new_settings_cookie(
            String::from(cookie),
            config.long_cookie_expire as i64,
        )), //cookie_raw = Some(String::from(cookie)),
        Err(error) => Err(Error::Other(error.to_string())),
    }
}

#[macro_export]
macro_rules! srender {
    ($render:expr) => {
        async move { StdResponse::Ok($render.await?) }
    };
}

/// Silly thing to limit a route by a single flag present (must be i8)
#[macro_export]
macro_rules! qflag {
    ($flag:ident, $req:expr) => {{
        #[allow(dead_code)]
        #[derive(serde::Deserialize)]
        struct LocalQueryParam {
            $flag: i8,
        }

        let mut result = false;
        if let Some(query) = $req.uri().query() {
            let r = serde_urlencoded::from_str::<LocalQueryParam>(query);
            if r.is_ok() {
                result = true;
            }
        }

        result
    }};
}

// Another silly thing for multi-route endpoints, parsing the form is always the same and we have to
// do it a million times.
#[macro_export]
macro_rules! parseform {
    ($wrapper:expr, $wrapped:ty, $req:expr) => {
        match Form::<$wrapped>::from_request($req, &()).await {
            Ok(Form(form)) => Ok($wrapper(form)),
            Err(e) => Err(e.into_response()),
        }
    };
}

#[async_trait]
impl FromRequestParts<Arc<GlobalState>> for RequestContext {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &Arc<GlobalState>,
    ) -> Result<Self, Self::Rejection> {
        use axum::RequestPartsExt;
        let cookies = parts
            .extract::<Cookies>()
            .await
            .map_err(|err| Self::Rejection::Other(err.1.to_string()))?;
        let full_uri = parts.extract::<axum::http::Uri>().await.unwrap(); //Infallible?
        let path = full_uri.path();

        let config_raw = cookies
            .get(SETTINGSCOOKIE)
            .and_then(|c| Some(c.value().to_string()));
        RequestContext::generate(state.clone(), path, config_raw).await
    }
}

// ---------------------------------
//         FORUM STUFF
// ---------------------------------

#[allow(dead_code)]
#[derive(serde::Deserialize, Debug)]
pub struct ForumFullQuery {
    fcid: Option<i64>, //Lots of options because of legacy junk
    ftid: Option<i64>,
    fpid: Option<i64>,
    page: Option<i32>,
}

pub async fn forum_get(
    context: RequestContext,
    Query(query): Query<ForumFullQuery>,
) -> StdResponse {
    //Order goes from most precise to least
    if let Some(fpid) = query.fpid {
        crate::pages::redirect::get_fpid_redirect(context.page_context, fpid).await
    } else if let Some(ftid) = query.ftid {
        crate::pages::redirect::get_ftid_redirect(context.page_context, ftid, query.page).await
    } else if let Some(fcid) = query.fcid {
        crate::pages::forum_category::get_fcid_render(
            context.page_context,
            fcid,
            context.global_state.config.default_display_threads,
            query.page,
        )
        .await
    } else {
        //This is main forum display, usually what we want (but unfortunately last)
        crate::pages::forum_main::get_render(
            context.page_context,
            &context.global_state.config.forum_category_order,
        )
        .await
    }
}
