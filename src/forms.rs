#![allow(non_snake_case)]

use rocket::form::FromForm;
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