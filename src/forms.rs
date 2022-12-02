#![allow(non_snake_case)]

use rocket::{form::FromForm, fs::TempFile};
use serde::Serialize;

#[derive(Serialize, FromForm)]
pub struct Login<'a>
{
    pub username: &'a str,
    pub password: &'a str,
    pub long_session : bool  //This is from the form itself, just a checkbox
}

#[derive(Serialize, FromForm)]
pub struct Register<'a>
{
    pub username: &'a str,
    pub password: &'a str,
    pub email: &'a str 
}

#[derive(Serialize, FromForm)]
pub struct RegisterConfirm<'a>
{
    pub email: &'a str,
    pub key: &'a str
}

#[derive(Serialize, FromForm)]
pub struct RegisterResend<'a>
{
    pub email: &'a str
}

#[derive(Serialize, FromForm)]
pub struct LoginRecover<'a>
{
    pub email: &'a str
}

#[derive(Serialize, FromForm)]
pub struct UserSensitive<'a>
{
    pub username: Option<&'a str>,
    pub password: Option<&'a str>,
    pub email: Option<&'a str>,
    pub currentPassword: &'a str 
}

#[derive(Serialize, FromForm, Debug, Clone)]
pub struct ImageBrowseSearch<'a>
{
    #[field(default = 1i32, validate=range(1..=3))]
    pub size: i32,
    pub global: bool,
    pub oldest: bool,
    #[field(default = 0, validate=range(0..))]
    pub page: i32,
    pub preview: Option<&'a str>,
}

impl<'a> ImageBrowseSearch<'a> {
    pub fn new() -> Self {
        ImageBrowseSearch { size: 1, global: false, oldest: false, page: 0, preview: None }
    }
}

#[derive(FromForm, Debug)] //We're not sending this directly to the API so it's fine?
pub struct FileUpload<'a>
{
    pub file: TempFile<'a>
}

#[derive(FromForm, Debug)]
pub struct UserUpdate<'a>
{
    pub username: &'a str,
    pub avatar: &'a str,
    //pub special: &'a str,
    //#[field(default = false)]
    //pub admin: bool
}