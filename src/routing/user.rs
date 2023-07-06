use axum::{async_trait, extract::{FromRequest, Path}, Form, response::IntoResponse};

use crate::{state::RequestContext, qflag, parseform};

use super::{StdResponse, missing_type_response};


//User is a multi-route, meaning multiple things can be posted to it. But these are all mostly
//admin things...
pub enum UserPost {
    Ban(common::forms::BanForm),
    Unban(common::forms::UnbanForm),
    UserInfo(common::forms::UserUpdate),
}

#[async_trait]
impl<B, S> FromRequest<S, B> for UserPost
where 
    B: Send + 'static,
    S: Send + Sync,
    Form<common::forms::BanForm>: FromRequest<(), B>,
    Form<common::forms::UnbanForm>: FromRequest<(), B>,
    Form<common::forms::UserUpdate>: FromRequest<(), B>,
{
    type Rejection = axum::response::Response;

    async fn from_request(req: axum::http::Request<B>, _state: &S) -> Result<Self, Self::Rejection> 
    {
        //Post is either recover or regular login
        if qflag!(ban, req) {
            parseform!(UserPost::Ban, common::forms::BanForm, req)
        }
        else if  qflag!(unban, req) {
            parseform!(UserPost::Unban, common::forms::UnbanForm, req)
        }
        else if qflag!(userinfo, req) {
            parseform!(UserPost::UserInfo, common::forms::UserUpdate, req)
        }
        else {
            Err(missing_type_response())
        }
    }
}

pub async fn user_post(context: RequestContext, Path(username): Path<String>, post: UserPost) -> StdResponse
{
    match post {
        UserPost::Ban(form) => {
            pages::user::post_ban(context.page_context, username, form).await
        },
        UserPost::Unban(form) => {
            pages::user::post_unban(context.page_context, username, form).await
        },
        UserPost::UserInfo(form) => {
            pages::user::post_userinfo(context.page_context, username, form).await
        },
    }
}
