use bbcode::BBCode;

use super::*;

pub fn render(data: MainLayoutData, bbcode: &BBCode, text: Option<String>) -> String {
    html!{
        (DOCTYPE)
        html lang=(data.user_config.language) {
            head {
                (basic_meta(&data.config))
                title { "SmileBASIC Source BBCode Preview" }
                meta name="description" content="Show the rendered bbcode";
                (style(&data.config, "/base.css"))
                (script(&data.config, "/base.js"))
                (style(&data.config, "/forpage/bbcodepreview.css"))
            }
            //This is meant to go in an iframe, so it will use up the whole space
            body {
                @if let Some(text) = text {
                    div."content bbcode" { (PreEscaped(bbcode.parse(&text))) }
                }
                @else {
                    form method="POST" action=(data.current_path) {
                        textarea placeholder="Enter text to test here" name="text"{}
                        input type="submit" value="Test";
                    }
                }
            }
        }
    }.into_string()
}