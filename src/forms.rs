use rocket::form::FromForm;
use serde::Serialize;

#[derive(Serialize, FromForm)]
pub struct Login<'a>
{
    username: &'a str,
    password: &'a str
}
