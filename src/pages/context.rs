use crate::layout::*;
// use crate::links::*;
// use crate::response::*;
// use crate::Config;

use bbscope::*;
//use std::sync::Arc;

// pub struct GlobalState {
//     pub link_config: LinkConfig,
//     pub bbcode: BBCode,
//     pub config: Config,
// }

/// A basic context for use in page rendering. Even if a page doesn't strictly need all
/// the items inside this context, it just makes it easier to pass them all to every page
/// render consistently. However, do NOT use this on the baseline rendering functions!
pub struct PageContext {
    pub layout_data: MainLayoutData,
    //pub api_context: endpoints::ApiContext,
    pub bbcode: BBCode,
    pub dbcon: rusqlite::Connection,
}

// fn open_dbcon(db_path: &str) -> Result<rusqlite::Connection, Error> {
//     match rusqlite::Connection::open(db_path) {
//         Ok(conn) => Ok(conn),
//         Err(_) => Err(Error::Other(format!("Failed to open database"))),
//     }
// }

//fn open_dbcon() -> Result
//let dbcon = rusqlite::Connection::open(state.config.db_file)?;

// impl PageContext {
//     pub async fn generate(
//         state: Arc<GlobalState>,
//         path: &str,
//         config_raw: Option<String>,
//     ) -> Result<Self, Error> {
//         //let context = ApiContext::new(state.config.api_endpoint.clone());
//
//         let user_config = if let Some(config) = config_raw {
//             serde_json::from_str::<UserConfig>(&config)?
//         } else {
//             UserConfig::default()
//         };
//
//         let layout_data = MainLayoutData {
//             links: state.link_config.clone(),
//             user_config, //Local settings
//             current_path: String::from(path),
//             override_nav_path: None,
//         };
//
//         let dbcon = open_dbcon(&state.config.db_file)?;
//
//         return Ok(PageContext {
//             layout_data,
//             //api_context: context,
//             bbcode: state.bbcode.clone(),
//             dbcon,
//         });
//     }
// }
