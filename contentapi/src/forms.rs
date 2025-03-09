use serde::Serialize;

use super::*;

// -----------------------------
// *     QUERY PARAMETERS      *
// -----------------------------

/// Query string sent to /file/raw to change thumbnail received
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct QueryImage {
    pub size: Option<i64>,
    pub crop: Option<bool>,
}

impl QueryImage {
    pub fn avatar(size: i64) -> Self {
        QueryImage {
            size: Some(size),
            crop: Some(true),
        }
    }
}
