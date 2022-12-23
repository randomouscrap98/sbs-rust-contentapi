use common::*;
use common::layout::*;
//use contentapi::*;
//use contentapi::conversion::*;
use maud::*;
//use serde::{Serialize, Deserialize};

pub fn render(data: MainLayoutData) -> String
{
    layout(&data, html!{
        section {
            @if let Some(user) = &data.user {
                @if user.admin {
                    p { "Coming soon?" }
                }
                else {
                    p."error" { "You must be an admin to use this page!" }
                }
            }
            @else {
                p."error" { "You must be logged in to use this page!" }
            }
        }
    }).into_string()
}
