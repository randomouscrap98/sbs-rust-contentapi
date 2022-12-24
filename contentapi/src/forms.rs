use serde::Serialize;

use super::*;

// -----------------------------
// *     QUERY PARAMETERS      *
// -----------------------------

/// Query string sent to /file/raw to change thumbnail received
#[derive(Serialize, Deserialize, Debug)]
pub struct QueryImage
{
    pub size: Option<i64>,
    pub crop: Option<bool>
}


// -----------------------
// *    ACTUAL FORMS     *
// -----------------------

#[derive(Serialize, Deserialize, Debug)]
pub struct Login
{
    pub username: String,
    pub password: String,
    pub expireSeconds: i64 
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Register
{
    pub username: String,
    pub password: String,
    pub email: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterConfirm
{
    pub email: String,
    pub key: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserSensitive
{
    pub password: Option<String>,
    pub email: Option<String>,
    pub currentPassword: String,
    pub currentEmail: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileUploadAsObject {
    pub object: Content,
    pub base64blob: String, //This could be a VERY LARGE string!!!
}