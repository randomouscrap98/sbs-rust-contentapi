use serde::{Serialize, Deserialize};

// ------------------------
// *     GENERIC FORMS    *
// ------------------------


#[derive(Serialize, Deserialize, Debug)]
pub struct EmailGeneric
{
    pub email: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BasicText
{
    pub text: String
}
