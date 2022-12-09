use serde::Serialize;

use super::*;

//#[derive(Debug)] //We're not sending this directly to the API so it's fine?
//pub struct FileUpload<'a>
//{
//    pub file: TempFile<'a>
//}

#[derive(Serialize, Debug)]
pub struct Login<'a>
{
    pub username: &'a str,
    pub password: &'a str,
    pub expireSeconds: i64 
}


#[derive(Serialize, Debug)]
pub struct Register<'a>
{
    pub username: &'a str,
    pub password: &'a str,
    pub email: &'a str 
}

#[derive(Serialize, Debug)]
pub struct RegisterConfirm<'a>
{
    pub email: &'a str,
    pub key: &'a str
}

#[derive(Serialize, Debug)]
pub struct EmailGeneric<'a>
{
    pub email: &'a str
}

#[derive(Serialize, Debug)]
pub struct UserSensitive<'a>
{
    //pub username: Option<&'a str>,
    pub password: Option<&'a str>,
    pub email: Option<&'a str>,
    pub currentPassword: &'a str,
    pub currentEmail: &'a str
}

#[derive(Serialize, Debug)]
pub struct FileUploadAsObject {
    pub object: Content,
    pub base64blob: String, //This could be a VERY LARGE string!!!
}