use super::*;
use common::layout::*;

pub fn render(data: MainLayoutData) -> String {
    layout(&data, html!{
        section {
            h1 {"Activity?? It'll be different"}
        }
    }).into_string()
}