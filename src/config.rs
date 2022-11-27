use serde::{Deserialize, Serialize};

//Just data we get from the rocket.toml file, this is general configuration you might want to use anywhere
#[derive(Deserialize, Clone)]
pub struct Config
{
    pub api_endpoint: String,
    pub http_root: String,
    pub token_cookie_key: String
}

////Safe data we always send to the templates. Mostly static data or basic calculations,
////should ALWAYS be fast (if you're adding anything slow, rethink)
//#[derive(Serialize)]
//pub struct BaseTemplateData
//{
//    http_root: String
//}

impl Config 
{
    pub fn get_endpoint(&self, endpoint: &str) -> String
    {
        let mut result = self.api_endpoint.clone();
        result.push_str(endpoint);
        result
    }
}

//pub fn get_base_template_data(config: &Config) -> BaseTemplateData
//{
//    BaseTemplateData {
//        http_root: config.http_root.clone()
//    }
//}
