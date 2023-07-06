use std::sync::Arc;

use axum::{async_trait, extract::{FromRequest, Query}, Form, RequestExt, body::{HttpBody, Body}, response::IntoResponse};
use tower_cookies::Cookies;

use crate::{state::{RequestContext, GlobalState}, qflag};

use super::{StdResponse, get_new_login_cookie};

//Login is a multi-route, meaning multiple things can be posted to it. I don't care if this is bad,
//it's just how sbs is designed, so we need an enum that can consume the body conditionally
//based on request data (in this case, query params)
pub enum LoginPost {
    Login(pages::login::Login),
    Recover(common::forms::EmailGeneric)
}

#[async_trait]
impl<B> FromRequest<Arc<GlobalState>, B> for LoginPost
where 
    B: Send + 'static,
    Form<pages::login::Login>: FromRequest<(), B>,
    Form<common::forms::EmailGeneric>: FromRequest<(), B>,
{
    type Rejection = common::response::Error;

    async fn from_request(req: axum::http::Request<B>, _state: &Arc<GlobalState>) -> Result<Self, Self::Rejection> 
    {
        //This is recovery
        //let parts = req.extract_parts();
        if qflag!(recover, req) {
            match Form::<common::forms::EmailGeneric>::from_request(req, &()).await
            {
                Ok(Form(form)) => Ok(LoginPost::Recover(form)),
                Err(_) => Err(Self::Rejection::User(String::from("Can't parse form")))
            }
        }
        else {
            match Form::<pages::login::Login>::from_request(req, &()).await
            {
                Ok(Form(form)) => Ok(LoginPost::Login(form)),
                Err(_) => Err(Self::Rejection::User(String::from("Can't parse form")))
            }
            //let Form(form) = Form::<pages::login::Login>::from_request(req, &())
            //    .await
            //    .map_err(|err| Self::Rejection::User(err.to_string()))?;
            //Ok(LoginPost::Login(form))
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

//qstruct!(LoginRecoverQuery, recover);
//async fn login_recover_email_post(context: RequestContext, _query: Query<LoginRecoverQuery>, Form(form) : Form<common::forms::EmailGeneric>) -> StdResponse
//{
//}
//
//async fn login_post(context: RequestContext, cookies: Cookies, Form(form): Form<pages::login::Login>) -> StdResponse
//{
//}
