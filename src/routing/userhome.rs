use axum::{async_trait, extract::FromRequest, Form, response::IntoResponse};
use tower_cookies::Cookies;

use crate::{state::RequestContext, qflag, parseform};

use super::{StdResponse, get_new_login_cookie};

    //// Primary endpoint: update regular user data
    //let userhome_post = warp::any()
    //    .and(warp::body::form::<common::forms::UserUpdate>())
    //    .and(state_filter.clone())
    //    .and_then(|form, context: RequestContext| 
    //        std_resp!(pages::userhome::post_info_render(pc!(context), form), context)
    //    ).boxed();

    //// Secondary endpoint: user bio updates
    //let userhome_bio_post = warp::any()
    //    .and(qflag!(bio)) 
    //    .and(warp::body::form::<common::forms::BasicPage>())
    //    .and(state_filter.clone())
    //    .and_then(|_query, form, context: RequestContext| 
    //        std_resp!(pages::userhome::post_bio_render(pc!(context), form), context)
    //    ).boxed();

    //// Tertiary endpoint: user sensitive updates
    //let userhome_sensitive_post = warp::any()
    //    .and(qflag!(sensitive)) 
    //    .and(warp::body::form::<contentapi::forms::UserSensitive>())
    //    .and(state_filter.clone())
    //    .and_then(|_query, form, context: RequestContext| 
    //        std_resp!(pages::userhome::post_sensitive_render(pc!(context), form), context) 
    //    ).boxed();

    //warp::post()
    //    .and(warp::path!("userhome"))
    //    .and(form_filter.clone())
    //    .and(userhome_bio_post.or(userhome_sensitive_post).or(userhome_post))
    //    .boxed()

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
