//use bbscope::BBCode;

use common::*;
use common::render::forum::render_content_nocontent;
use common::render::layout::*;
use maud::*;
use serde::Deserialize;

#[derive(Default, Deserialize, Debug)]
//#[serde(default)]
pub struct ContentPreviewForm {
    pub text: String,
    pub markup: Option<String>
}

pub fn render(mut context: PageContext, form: ContentPreviewForm) -> String 
{
    basic_skeleton(&context.layout_data, html! {
        title { "SmileBASIC Source Content Preview" }
        meta name="description" content="Show the rendered content (unless it's js rendering)";
        //(data.links.style("/forpage/bbcodepreview.css"))
    }, html! {
        (render_content_nocontent(form.text, form.markup, &mut context.bbcode))
    }).into_string()
}