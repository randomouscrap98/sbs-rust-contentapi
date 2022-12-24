use bbscope::BBCode;
use contentapi::endpoints::ApiContext;
use serde::{Serialize, Deserialize};


#[derive(Clone, Debug)]
pub struct LinkConfig {
    pub http_root: String,
    pub static_root: String,
    pub resource_root: String,
    pub file_root: String,
    pub file_upload_root: String,
    pub cache_bust: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct UserConfig {
    pub language: String,
    pub compact: bool,
    pub theme: String
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            language: String::from("en"),
            compact: false,
            theme: String::from("sbs")
        }
    }
}

#[derive(Debug)]
pub struct MainLayoutData {
    pub links: LinkConfig,     
    pub user_config: UserConfig,    
    pub current_path: String, 
    pub override_nav_path: Option<&'static str>,
    pub user: Option<contentapi::User>,
    pub user_token: Option<String>,
    pub about_api: contentapi::About, 
    pub raw_alert: Option<String>,

    #[cfg(feature = "profiling")]
    pub profiler: onestop::OneList<onestop::OneDuration>
}

/// A basic context for use in page rendering. Even if a page doesn't strictly need all
/// the items inside this context, it just makes it easier to pass them all to every page
/// render consistently. However, do NOT use this on the baseline rendering functions!
pub struct PageContext {
    pub layout_data: MainLayoutData,
    pub api_context: ApiContext,
    pub bbcode: BBCode
}
