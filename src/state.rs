use std::sync::Arc;

use bbscope::BBCode;
use contentapi::endpoints::{ApiContext};
use common::{LinkConfig, MainLayoutData, UserConfig, PageContext};
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
    pub bbcode: BBCode, //Clones are cheap?
    pub api_context: ApiContext,
    pub layout_data: MainLayoutData,

    #[cfg(feature = "profiling")]
    pub profiler: onestop::OneList<onestop::OneDuration>,
}

impl RequestContext {
    pub async fn generate(state: Arc<GlobalState>, path: FullPath, token: Option<String>, config_raw: Option<String>) -> 
        Result<Self, common::Error> 
    {
        #[cfg(feature = "profiling")]
        let profiler = onestop::OneList::<onestop::OneDuration>::new(); //One profiler per request

        #[cfg(feature = "profiling")]
        let context = ApiContext::new_with_profiler(
            state.config.api_endpoint.clone(), 
            token.clone(),
            profiler.clone()
        );

        #[cfg(not(feature = "profiling"))]
        let context = ApiContext::new(
            state.config.api_endpoint.clone(), 
            token.clone()
        );

        let user_config = if let Some(config) = config_raw {
            serde_json::from_str::<UserConfig>(&config)?
        }
        else {
            UserConfig::default()
        };

        let layout_data = MainLayoutData 
        {
            config: state.link_config.clone(),
            user_config, //Local settings
            current_path: String::from(path.as_str()),
            user: context.get_me_safe().await,
            user_token: token,
            about_api: context.get_about().await?,

            #[cfg(feature = "profiling")]
            profiler: profiler.clone()
        };

        #[cfg(feature = "profiling")]
        return Ok(RequestContext 
        {
            //Custom construct bbcode so we copy the matchers but NOT the profiler!
            bbcode: BBCode { matchers: state.bbcode.matchers.clone(), profiler: profiler.clone() },
            global_state: state,
            api_context: context,
            layout_data,
            profiler
        });

        #[cfg(not(feature = "profiling"))]
        return Ok(RequestContext 
        {
            bbcode: state.bbcode.clone(), 
            global_state: state,
            api_context: context,
            layout_data,
        });
    }
}

impl From<RequestContext> for PageContext {
    fn from(context: RequestContext) -> Self {
        Self {
            layout_data: context.layout_data,
            api_context: context.api_context,
            bbcode: context.bbcode
        } 
    }
}