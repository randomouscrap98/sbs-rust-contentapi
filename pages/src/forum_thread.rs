//use common::capi::get_forum_categories;
use common::capi::get_forum_category_by_id;
//use common::capi::get_msgid_contentid_by_fpid;
use common::capi::get_submission_categories;
use common::capi::get_thread_by_hash;
//use common::capi::IdOrHash;
use common::constants::SBSPageType;
use common::forum::*;
use common::pagination::*;
//use common::prefab::*;
use common::render::forum::*;
use common::render::layout::*;
use common::render::*;
use common::response::*;
use common::view::*;
use common::*;

use contentapi::conversion::*;
use contentapi::*;
use contentapi::{FullRequest, SpecialCount};

pub fn render(mut context: PageContext, config: PostsConfig) -> String {
    let mut meta = LayoutMeta {
        title: format!("SBS ⦁ {}", config.thread.name),
        description: short_description2(&config.thread),
        image: get_thumbnail_hash2(&config.thread).and_then(|h| {
            Some(
                context
                    .layout_data
                    .links
                    .image(&h, contentapi::QueryImage::Scaled300),
            )
        }),
        canonical: Some(
            context
                .layout_data
                .links
                .forum_thread_unsafe(&config.thread.hash),
        ),
    };

    //If the literal type is a forumthread, we generally want to pull text from posts
    if config.thread.literal_type == SBSPageType::FORUMTHREAD {
        if let Some(selected_id) = config.selected_post_id {
            if let Some(post) = config.posts.iter().find(|p| p.id == Some(selected_id)) {
                meta.description = short_post(post);
                meta.canonical = Some(
                    context
                        .layout_data
                        .links
                        .forum_post_unsafe(post.id.unwrap_or_default(), &config.thread.hash),
                );
            }
        } else if let Some(start) = config.start_num {
            if start == 1 {
                //We KNOW this is the first page, so we can actually give the post as the description!
                if let Some(post) = config.posts.get(0) {
                    meta.description = short_post(post);
                }
            }
        }
    }

    let main_page = render_mainview(&mut context, config);
    layout_with_meta(&context.layout_data, meta, main_page).into_string()
}

async fn render_thread(
    context: PageContext,
    hash: &str,
    pre_request: FullRequest,
    per_page: i32,
    page: Option<i32>,
) -> Result<Response, Error> {
    let mut page = page.unwrap_or(1) - 1; //we assume 1-based pages

    //Go lookup all the 'initial' data, which everything except posts and users
    let pre_result = context.api_context.post_request(&pre_request).await?;

    let thread = get_thread_by_hash(&context, hash)?
        .ok_or(Error::NotFound(String::from("Could not find thread!")))?;
    let category = get_forum_category_by_id(&context, thread.parent_id)?
        .ok_or(Error::NotFound(String::from("Could not find category!")))?;
    let thread_subcat_ids = get_tagged_categories2(&thread.values);
    let subcategories = get_submission_categories(&context, Some(thread_subcat_ids))?;
    //let thread_sub_categories= get_submission_categories(&thread);

    //Pull out and parse all that stupid data. It's fun using strongly typed languages!! maybe...
    //let mut threads_raw = cast_result_required::<Content>(&pre_result, THREADKEY)?;
    let selected_post = cast_result_safe::<Message>(&pre_result, PREMESSAGEKEY)?.pop();
    if let Some(message_index) =
        cast_result_safe::<SpecialCount>(&pre_result, PREMESSAGEINDEXKEY)?.pop()
    {
        //The index is the special count. This means we change the page given. If page wasn't already 0, we warn
        if page != 0 {
            println!(
                "Page was nonzero ({}) while there was a message index ({})",
                page, message_index.specialCount
            );
        }
        page = message_index.specialCount / per_page;
    }

    //There must be one category, and one thread, otherwise return 404
    // let thread = threads_raw
    //     .pop()
    //     .ok_or(Error::NotFound(String::from("Could not find thread!")))?;

    //Also I need some fields to exist.
    let thread_id = thread.id;
    let thread_create_uid = thread.create_user_id;
    let comment_count = thread.posts_count;
    // let thread_id = thread.id.ok_or(Error::Other(String::from(
    //     "Thread result did not have id field?!",
    // )))?;
    // let thread_create_uid = thread.createUserId.ok_or(Error::Other(String::from(
    //     "Thread result did not have createUserId field!",
    // )))?;
    // let comment_count = thread.commentCount.ok_or(Error::Other(String::from(
    //     "Thread result did not have commentCount field!",
    // )))?;

    let sequence_start = page * per_page;

    //OK NOW you can go lookup the posts, since we are sure about where in the postlist we want
    let after_request =
        get_finishpost_request(thread_id, vec![thread_create_uid], per_page, sequence_start);
    let after_result = context.api_context.post_request(&after_request).await?;

    //Pull the data out of THAT request
    let messages_raw = cast_result_required::<Message>(&after_result, "message")?;
    let related_raw = cast_result_required::<Message>(&after_result, "related")?;
    let users_raw = cast_result_required::<User>(&after_result, "user")?;

    //Construct before borrowing
    let path = vec![
        ForumPathItem::root(),
        ForumPathItem::from_category2(&category),
        ForumPathItem::from_thread2(&thread),
    ];
    //let thread_tags_ids = get_tagged_categories(&thread);
    // let mut full_thread = ForumThread::from_content(thread, &messages_raw)?;
    // full_thread.categories = Some(get_submission_categories(
    //     &mut context,
    //     Some(thread_tags_ids),
    // )?);
    let post_config = PostsConfig::thread_mode(
        thread,
        subcategories,
        messages_raw,
        map_messages(related_raw),
        map_users(users_raw),
        path,
        get_pagelist(comment_count as i32, per_page, page),
        1 + per_page * page,
        selected_post.and_then(|m| m.id),
    );
    // if post_config.thread.thread.literalType.as_deref() == Some(SBSPageType::DOCUMENTATION) {
    //     post_config.docs_content = Some(get_all_documentation(&mut context.api_context).await?);
    // }
    Ok(Response::Render(render(context, post_config)))
}

/// The normal endpoint for listing a thread
pub async fn get_hash_render(
    context: PageContext,
    hash: String,
    per_page: i32,
    page: Option<i32>,
) -> Result<Response, Error> {
    let hurgh = hash.clone();
    render_thread(
        context,
        &hurgh,
        get_prepost_request(None, Some(hash)),
        per_page,
        page,
    )
    .await
}

/// The normal endpoint for pinpointing a post
pub async fn get_hash_postid_render(
    context: PageContext,
    hash: String,
    post_id: i64,
    per_page: i32,
) -> Result<Response, Error> {
    let hurgh = hash.clone();
    render_thread(
        context,
        &hurgh,
        get_prepost_request(Some(post_id), Some(hash)),
        per_page,
        None,
    )
    .await
}

pub async fn get_ftid_render(
    context: PageContext,
    ftid: i64,
    //per_page: i32,
    page: Option<i32>,
) -> Result<Response, Error> {
    if let Some(hash) = capi::get_contenthash_by_ftid(&context, ftid)? {
        let mut url = context.layout_data.links.forum_thread_unsafe(&hash);
        if let Some(page) = page {
            url.push_str(&format!("?page={}", page))
        }
        Ok(Response::Redirect(url))
    } else {
        Err(Error::NotFound(format!("Can't find thread {}", ftid)))
    }
    // render_thread(
    //     context,
    //     get_prepost_request(None, Some(ftid), None),
    //     per_page,
    //     page,
    // )
    // .await
}

//Most old links may be to posts directly? idk
pub async fn get_fpid_render(
    context: PageContext,
    fpid: i64,
    //per_page: i32,
) -> Result<Response, Error> {
    // These are redirects now, 2025
    if let Some((msgid, hash)) = capi::get_msgid_contenthash_by_fpid(&context, fpid)? {
        let url = context.layout_data.links.forum_post_unsafe(msgid, &hash);
        Ok(Response::Redirect(url))
    } else {
        Err(Error::NotFound(format!("Can't find post {}", fpid)))
    }

    // render_thread(
    //     context,
    //     get_prepost_request(Some(fpid), None, None, None),
    //     per_page,
    //     None,
    // )
    // .await
}
