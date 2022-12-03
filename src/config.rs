use serde::Deserialize;

//Just data we get from the rocket.toml file, this is general configuration you might want to use anywhere
#[derive(Deserialize, Clone)]
pub struct Config
{
    pub api_endpoint: String,
    pub http_root: String,
    pub api_fileraw : String,
    //pub api_fileupload : String,
    pub token_cookie_key: String,
    pub default_cookie_expire: i32,
    pub long_cookie_expire: i32,
    pub default_imagebrowser_count: i32,
    pub default_category_threads : i32,
    pub default_display_threads : i32,
    pub forum_category_order: Vec<String>
}

impl Config 
{
    pub fn get_endpoint(&self, endpoint: &str) -> String
    {
        let mut result = self.api_endpoint.clone();
        result.push_str(endpoint);
        result
    }
}
