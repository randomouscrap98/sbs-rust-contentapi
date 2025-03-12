use common::forum::*;
use common::render::layout::*;
use common::response::*;
use common::*;
use maud::*;

pub fn render(data: MainLayoutData, categories: Vec<ForumCategory2>) -> String {
    layout(&data, html!{
        (data.links.style("/forpage/forum.css"))
        section { h1 { "Forum Topics" } }
        section {
            @for (index, category_container) in categories.iter().enumerate() {
                div."category" {
                    div."categoryinfo" {
                        h1 { a."flatlink" title={"ID: " (category_container.id)} href=(data.links.forum_category_unsafe(&category_container.hash)) {(category_container.name)} }
                        p."aside" {(category_container.description)}
                    }
                    div."foruminfo aside mediumseparate" {
                        div { b{"Threads: "} (category_container.threads_count) }
                    }
                }
                @if index < categories.len() - 1 {
                    hr;
                }
            }
        }
    }).into_string()
}

pub async fn get_render(context: PageContext, order: &Vec<String>) -> Result<Response, Error> {
    let mut categories = get_categories(&context, None)?;

    //Sort the categories by their name AGAINST the default list in the config. So, it should sort the categories
    //by the order defined in the config, with stuff not present going at the end. Tiebreakers are resolved alphabetically
    categories.sort_by_key(|category| {
        //Nicole made this a tuple so tiebreakers are sorted alphabetically, which is coool
        (
            order
                .iter()
                .position(|prefix| category.name.starts_with(prefix))
                .unwrap_or(usize::MAX),
            category.name.clone(),
        )
    });

    Ok(Response::Render(render(context.layout_data, categories)))
}
