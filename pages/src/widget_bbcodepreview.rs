use bbscope::BBCode;

use common::*;
use common::layout::*;
use maud::*;

pub fn render(data: MainLayoutData, bbcode: &BBCode, text: Option<String>) -> String 
{
    basic_skeleton(&data, html! {
        title { "SmileBASIC Source BBCode Preview" }
        meta name="description" content="Show the rendered bbcode";
        (style(&data.config, "/forpage/bbcodepreview.css"))
    }, html! {
        @if let Some(text) = text {
            div."content bbcode" { (PreEscaped(bbcode.parse(&text))) }
        }
        @else {
            form method="POST" action={(data.config.http_root)"/widget/bbcodepreview"} {
                textarea placeholder="Enter text to test here" name="text"{}
                input type="submit" value="Test";
            }
        }
    }).into_string()
}