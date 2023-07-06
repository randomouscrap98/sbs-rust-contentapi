use axum::{async_trait, extract::FromRequest, Form, response::IntoResponse};
use tower_cookies::Cookies;

use crate::{state::RequestContext, qflag, parseform};

use super::{StdResponse, get_new_login_cookie};

//Login is a multi-route, meaning multiple things can be posted to it. I don't care if this is bad,
//it's just how sbs is designed, so we need an enum that can consume the body conditionally
//based on request data (in this case, query params)
pub enum LoginPost {
    Login(pages::login::Login),
    Recover(common::forms::EmailGeneric)
}

#[async_trait]
impl<B, S> FromRequest<S, B> for LoginPost
where 
    B: Send + 'static,
    S: Send + Sync,
    Form<pages::login::Login>: FromRequest<(), B>,
    Form<common::forms::EmailGeneric>: FromRequest<(), B>,
{
    type Rejection = axum::response::Response;

    async fn from_request(req: axum::http::Request<B>, _state: &S) -> Result<Self, Self::Rejection> 
    {
        //Post is either recover or regular login
        if qflag!(recover, req) {
            parseform!(LoginPost::Recover, common::forms::EmailGeneric, req)
        }
        else {
            parseform!(LoginPost::Login, pages::login::Login, req)
        }
    }
}

pub async fn login_post(context: RequestContext, cookies: Cookies, post: LoginPost) -> StdResponse
{
    match post {
        LoginPost::Login(form) => {
            let login = form.to_api_login(
                context.global_state.config.default_cookie_expire, 
                context.global_state.config.long_cookie_expire);
            let (response,token) = pages::login::post_login_render(context.page_context, &login).await;
            if let Some(token) = token { cookies.add(get_new_login_cookie(token, login.expireSeconds)); }
            StdResponse::Ok(response)
        },
        LoginPost::Recover(form) => {
            let response = pages::login::post_login_recover(context.page_context, &form).await;
            StdResponse::Ok(response)
        }
    }
}
