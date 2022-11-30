use rocket::form::FromForm;
use serde::Serialize;

#[derive(Serialize, FromForm)]
pub struct Login<'a>
{
    pub username: &'a str,
    pub password: &'a str
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