use axum::{async_trait, extract::FromRequest, response::IntoResponse, Form};

use crate::{parseform, qflag, state::RequestContext};

use super::{missing_type_response, StdResponse};

//Admin is a multi-route, meaning multiple things can be posted to it.
pub enum AdminPost {
    Frontpage(common::forms::BasicPage),
    DocsCustom(common::forms::BasicPage),
    Alert(common::forms::BasicPage),
}

#[async_trait]
impl<B, S> FromRequest<S, B> for AdminPost
where
    B: Send + 'static,
    S: Send + Sync,
    Form<common::forms::BasicPage>: FromRequest<(), B>,
{
    type Rejection = axum::response::Response;

    async fn from_request(
        req: axum::http::Request<B>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        if qflag!(frontpage, req) {
            parseform!(AdminPost::Frontpage, common::forms::BasicPage, req)
        } else if qflag!(docscustom, req) {
            parseform!(AdminPost::DocsCustom, common::forms::BasicPage, req)
        } else if qflag!(alert, req) {
            parseform!(AdminPost::Alert, common::forms::BasicPage, req)
        } else {
            Err(missing_type_response())
        }
    }
}

pub async fn admin_post(context: RequestContext, post: AdminPost) -> StdResponse {
    match post {
        AdminPost::Frontpage(form) => {
            pages::admin::post_frontpage(context.page_context, form).await
        }
        AdminPost::DocsCustom(form) => {
            pages::admin::post_docscustom(context.page_context, form).await
        }
        AdminPost::Alert(form) => pages::admin::post_alert(context.page_context, form).await,
    }
}
