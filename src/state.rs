use std::sync::Arc;

use crate::layout::*;
use crate::links::*;
use crate::pages::context::*;
use crate::response::*;

use bbscope::BBCode;

use crate::Config;

/// The unchanging configuration for the current runtime. Mostly values read from
/// config, but some other constructed data too
pub struct GlobalState {
    pub link_config: LinkConfig,
    pub bbcode: BBCode,
    pub config: Config,
}

/// A context generated for each request. Even if the request doesn't need all the data,
/// this context is generated. The global_state is pretty cheap, and nearly all pages
/// require the api_about in MainLayoutData, which requires the api_context.
pub struct RequestContext {
    pub global_state: Arc<GlobalState>,
    pub page_context: PageContext,
}

fn open_dbcon(db_path: &str) -> Result<rusqlite::Connection, Error> {
    match rusqlite::Connection::open(db_path) {
        Ok(conn) => Ok(conn),
        Err(_) => Err(Error::Other(format!("Failed to open database"))),
    }
}

impl RequestContext {
    pub async fn generate(
        state: Arc<GlobalState>,
        path: &str,
        config_raw: Option<String>,
    ) -> Result<Self, Error> {
        let user_config = if let Some(config) = config_raw {
            serde_json::from_str::<UserConfig>(&config)?
        } else {
            UserConfig::default()
        };

        let layout_data = MainLayoutData {
            links: state.link_config.clone(),
            user_config, //Local settings
            current_path: String::from(path),
            override_nav_path: None,
        };

        let dbcon = open_dbcon(&state.config.db_file)?;

        return Ok(RequestContext {
            page_context: PageContext {
                layout_data,
                //api_context: context,
                bbcode: state.bbcode.clone(),
                dbcon,
            },
            global_state: state,
        });
    }
}
