use serde::Serialize;

use super::*;

//#[derive(Debug)] //We're not sending this directly to the API so it's fine?
//pub struct FileUpload<'a>
//{
//    pub file: TempFile<'a>
//}

#[derive(Serialize, Debug)]
pub struct Login
{
    pub username: String,
    pub password: String,
    pub expireSeconds: i64 
}


#[derive(Serialize, Debug)]
pub struct Register
{
    pub username: String,
    pub password: String,
    pub email: String
}

#[derive(Serialize, Debug)]
pub struct RegisterConfirm
{
    pub email: String,
    pub key: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmailGeneric
{
    pub email: String
}

#[derive(Serialize, Debug)]
pub struct UserSensitive
{
    //pub username: Option<&'a str>,
    pub password: Option<String>,
    pub email: Option<String>,
    pub currentPassword: String,
    pub currentEmail: String
}

#[derive(Serialize, Debug)]
pub struct FileUploadAsObject {
    pub object: Content,
    pub base64blob: String, //This could be a VERY LARGE string!!!
}