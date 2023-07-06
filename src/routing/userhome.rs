use axum::{async_trait, extract::FromRequest, Form, response::IntoResponse};

use crate::{state::RequestContext, qflag, parseform};

use super::StdResponse;

//Userhome is a multi-route, meaning multiple things can be posted to it.
pub enum UserhomePost {
    UserUpdate(common::forms::UserUpdate),
    BioUpdate(common::forms::BasicPage),
    SensitiveUpdate(contentapi::forms::UserSensitive),
}

#[async_trait]
impl<B, S> FromRequest<S, B> for UserhomePost
where 
    B: Send + 'static,
    S: Send + Sync,
    Form<common::forms::UserUpdate>: FromRequest<(), B>,
    Form<common::forms::BasicPage>: FromRequest<(), B>,
    Form<contentapi::forms::UserSensitive>: FromRequest<(), B>,
{
    type Rejection = axum::response::Response;

    async fn from_request(req: axum::http::Request<B>, _state: &S) -> Result<Self, Self::Rejection> 
    {
        //Post is either recover or regular login
        if qflag!(bio, req) {
            parseform!(UserhomePost::BioUpdate, common::forms::BasicPage, req)
        }
        else if  qflag!(sensitive, req) {
            parseform!(UserhomePost::SensitiveUpdate, contentapi::forms::UserSensitive, req)
        }
        else {
            parseform!(UserhomePost::UserUpdate, common::forms::UserUpdate, req)
        }
    }
}

pub async fn userhome_post(context: RequestContext, post: UserhomePost) -> StdResponse
{
    match post {
        UserhomePost::UserUpdate(form) => {
            pages::userhome::post_info_render(context.page_context, form).await
        },
        UserhomePost::BioUpdate(form) => {
            pages::userhome::post_bio_render(context.page_context, form).await
        },
        UserhomePost::SensitiveUpdate(form) => {
            pages::userhome::post_sensitive_render(context.page_context, form).await
        },
    }
}
