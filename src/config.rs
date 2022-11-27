use serde::Deserialize;

//Just data we get from the rocket.toml file, this is general configuration you might want to use anywhere
#[derive(Deserialize, Clone)]
pub struct Config
{
    pub api_endpoint: String,
    pub http_root: String,
    pub token_cookie_key: String
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
