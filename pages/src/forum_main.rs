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
    // //The request which we will spend the entire function building
    // let mut request = FullRequest::new();

    // let mut real_query = String::from("!notdeleted()");

    // if let Some(hash) = hash {
    //     // NOTE: I don't remember why I don't check for the category type when doing this
    //     // query. It might not be important and maybe you could always check for type, just be careful
    //     add_value!(request, "hash", hash);
    //     real_query.push_str(" and hash = @hash");
    // } else {
    //     //This is the "general" case, where yes, we actually do want to limit to categories. Otherwise,
    //     //if you pass a hash... it'll just work, regardless if it's a category or not.
    //     add_value!(request, "category_literals", FORUMCATEGORYTYPES);
    //     real_query.push_str(" and literalType in @category_literals");

    //     if let Some(fcid) = fcid {
    //         add_value!(request, "fcid_key", vec!["fcid"]);
    //         add_value!(request, "fcid", vec![fcid]);
    //         real_query.push_str(" and !valuein(@fcid_key, @fcid)");
    //     }
    // }

    // let mut category_request = build_request!(
    //     RequestType::content,
    //     String::from(CATEGORYFIELDS),
    //     real_query
    // );
    // category_request.name = Some(String::from(CATEGORYKEY));
    // request.requests.push(category_request);

    // Categories:
    // - not deleted
    // - public
    // - thread count (will produce subqueries)
    // - specific type (FORUMCATEGORYTYPES)
    //   add_value!(request, "category_literals", FORUMCATEGORYTYPES);
    //   real_query.push_str(" and literalType in @category_literals");
    //context.dbcon.;
    //First request: just get categories
    //let request = get_category_request(None, None);
    // let category_result = context.api_context.post_request(&request).await?;
    // let mut categories_cleaned = CleanedPreCategory::from_many(cast_result_required::<Content>(
    //     &category_result,
    //     CATEGORYKEY,
    // )?)?;

    // let categories =
    //     build_categories_with_threads(context.api_context, categories_cleaned, show_threads, 0)
    //         .await?;
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
