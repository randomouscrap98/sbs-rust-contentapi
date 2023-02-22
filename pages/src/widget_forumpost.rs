//use common::*;
//use common::forms::*;
//use common::render::forum::post_textbox;
//use common::render::layout::*;
//use maud::*;
//
//pub fn render(data: MainLayoutData, form: PostForm) -> String 
//{
//    let submit_value = 
//        if form.id == 0 {
//            if let Some(_reply_id) = form.reply_id {
//                "Post reply"
//            }
//            else {
//                "Submit post"
//            }
//        }
//        else {
//            "Edit post"
//        };
//
//    basic_skeleton(&data, html! {
//        title { "SmileBASIC Source Post Form" }
//        meta name="description" content="The form to make posts";
//    }, html!{
//        form."editor" #"postedit_form" method="POST" action=(data.links.forum_post_editor()) {
//            input #"postedit_content_id" type="hidden" name="content_id" value=(form.content_id);
//            input #"postedit_id" type="hidden" name="id" value=(form.id);
//            @if let Some(reply_id) = form.reply_id {
//                input #"postedit_reply_id" type="hidden" name="reply_id" value=(reply_id);
//            }
//            label for="postedit_post" {"Post:"}
//            (post_textbox(Some("postedit_post"), Some("post"), Some(&form.post)))
//            input type="submit" value=(submit_value);
//        }
//    }).into_string()
//}