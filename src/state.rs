use std::sync::Arc;

use bbcode::BBCode;
use contentapi::endpoints::{ApiContext, ApiError};
use pages::{LinkConfig, MainLayoutData, UserConfig};
use warp::path::FullPath;

use crate::Config;


/// The unchanging configuration for the current runtime. Mostly values read from 
/// config, but some other constructed data too
pub struct GlobalState {
    pub link_config: LinkConfig,
    pub bbcode: BBCode,
    pub config: Config
}

/// A context generated for each request. Even if the request doesn't need all the data,
/// this context is generated. The global_state is pretty cheap, and nearly all pages 
/// require the api_about in MainLayoutData, which requires the api_context.
pub struct RequestContext {
    pub global_state: Arc<GlobalState>,
    pub api_context: ApiContext,
    pub layout_data: MainLayoutData
}

impl RequestContext {
    pub async fn generate(state: Arc<GlobalState>, path: FullPath, token: Option<String>) -> Result<Self, ApiError> {
        let context = ApiContext::new(state.config.api_endpoint.clone(), token.clone());
        let layout_data = MainLayoutData {
            config: state.link_config.clone(),
            user_config: UserConfig::default(),
            current_path: String::from(path.as_str()),
            user: context.get_me_safe().await,
            user_token: token,
            about_api: context.get_about().await?
        };
        Ok(RequestContext {
            global_state: state,
            api_context: context,
            layout_data
        })
    }
}
