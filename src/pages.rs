pub mod fragments;

use crate::api;
use serde_urlencoded;

pub trait LinkConfig {
    fn get_http_root(&self) -> &str;
    fn get_static_root(&self) -> &str;
    fn get_resource_root(&self) -> &str;
    fn get_file_root(&self) -> &str;
}

pub trait UserConfig {
    fn get_language(&self) -> &str { //this isn't available yet
        "en"
    }
}

pub fn get_image_link(config: &impl LinkConfig, hash: &str, size: i32, crop: bool) -> String {
    let query = api::QueryImage { 
        size : Some(size as i64),
        crop : Some(crop) 
    };
    match serde_urlencoded::to_string(&query) {
        Ok(querystring) => format!("{}/{}?{}", config.get_file_root(), hash, querystring),
        Err(error) => {
            println!("Serde_qs failed? Not printing link for {}. Error: {}", hash, error);
            format!("#ERRORFOR-{}",hash)
        }
    }
}