use super::*;

pub fn render(data: MainLayoutData) -> String {
    layout(&data, html!{
        section {
            h1 { "Browse is search"}
            p { "Search may be simultaneously more powerful and less powerful than before"}
        }
    }).into_string()
}