use common::*;
use common::layout::*;
use maud::html;

pub fn render(data: MainLayoutData) -> String {
    layout(&data, html!{
        section {
            h1 {"Activity?? It'll be different"}
        }
    }).into_string()
}