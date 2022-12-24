use bbscope::BBCode;

use common::*;
use common::render::layout::*;
use maud::*;

pub fn render(data: MainLayoutData, bbcode: &BBCode, text: Option<String>) -> String 
{
    basic_skeleton(&data, html! {
        title { "SmileBASIC Source BBCode Preview" }
        meta name="description" content="Show the rendered bbcode";
        (data.links.style("/forpage/bbcodepreview.css"))
    }, html! {
        @if let Some(text) = text {
            div."content bbcode" { (PreEscaped(bbcode.parse(&text))) }
        }
        @else {
            form method="POST" action={(data.links.http_root)"/widget/bbcodepreview"} {
                textarea placeholder="Enter text to test here" name="text"{}
                input type="submit" value="Test";
            }
        }
    }).into_string()
}