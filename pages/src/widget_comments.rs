use super::*;

//This isn't JUST 'comments', it's anything that displays comment-like structures, INCLUDING forums!

pub fn post_item(context: &mut PageContext, post: &Message, thread: &Content, selected_post_id: Option<i64>, 
    users: &HashMap<i64, User>, sequence: i32) -> Markup 
{
    let config = &context.layout_data.config;
    let bbcode = &mut context.bbcode;
    let user = user_or_default(users.get(&post.createUserId.unwrap_or(0)));
    let mut class = String::from("post");
    if selected_post_id == post.id { class.push_str(" current") }
    html! {
        div.(class) #{"post_"(i(&post.id))} {
            div."postleft" {
                img."avatar" src=(image_link(config, &user.avatar, 100, true)); 
            }
            div."postright" {
                div."postheader" {
                    a."flatlink username" href=(user_link(config, &user)) { (&user.username) } 
                    a."sequence" title=(i(&post.id)) href=(forum_post_link(config, post, thread)){ "#" (sequence) } 
                }
                @if let Some(text) = &post.text {
                    div."content bbcode" { (PreEscaped(bbcode.parse_profiled_opt(text, format!("post-{}",i(&post.id))))) }
                }
                div."postfooter" {
                    div."history" {
                        time."aside" datetime=(d(&post.createDate)) { (timeago_o(&post.createDate)) } 
                        @if let Some(edit_user_id) = post.editUserId {
                            time."aside" datetime=(d(&post.editDate)) { 
                                "Edited "(timeago_o(&post.editDate))" by "
                                @if let Some(edit_user) = users.get(&edit_user_id) {
                                    a."flatlink" href=(user_link(config,&edit_user)){ (&edit_user.username) }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}