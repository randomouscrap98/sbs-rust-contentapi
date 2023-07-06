use axum::{async_trait, extract::FromRequest, Form, response::IntoResponse};
use tower_cookies::Cookies;

use crate::{state::RequestContext, qflag, parseform};

use super::{StdResponse, get_new_login_cookie};

//RegisterConfirm is a multi-route, meaning multiple things can be posted to it
pub enum RegisterConfirmPost {
    Confirm(contentapi::forms::RegisterConfirm),
    Resend(common::forms::EmailGeneric)
}

#[async_trait]
impl<B, S> FromRequest<S, B> for RegisterConfirmPost
where 
    B: Send + 'static,
    S: Send + Sync,
    Form<contentapi::forms::RegisterConfirm>: FromRequest<(), B>,
    Form<common::forms::EmailGeneric>: FromRequest<(), B>,
{
    type Rejection = axum::response::Response;

    async fn from_request(req: axum::http::Request<B>, _state: &S) -> Result<Self, Self::Rejection> 
    {
        //Post is either confirmation or resend
        if qflag!(resend, req) {
            parseform!(RegisterConfirmPost::Resend, common::forms::EmailGeneric, req)
        }
        else {
            parseform!(RegisterConfirmPost::Confirm, contentapi::forms::RegisterConfirm, req)
        }
    }
}

pub async fn registerconfirm_post(context: RequestContext, cookies: Cookies, post: RegisterConfirmPost) -> StdResponse
{
    match post {
        RegisterConfirmPost::Confirm(form) => {
            let (response,token) = pages::registerconfirm::post_render(context.page_context, &form).await;
            if let Some(token) = token { cookies.add(get_new_login_cookie(token, context.global_state.config.default_cookie_expire as i64)); }
            StdResponse::Ok(response)
        },
        RegisterConfirmPost::Resend(form) => {
            pages::registerconfirm::post_email_render(context.page_context, &form).await
        }
    }
}
