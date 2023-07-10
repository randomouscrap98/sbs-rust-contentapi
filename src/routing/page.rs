
/// Existence of parameters indicates which kind of form to generate
#[derive(serde::Deserialize, Debug)]
pub struct PageEditParameter { 
    pub mode: Option<String>,
    pub page: Option<String>
}
