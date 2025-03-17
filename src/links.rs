//use super::*;
//use contentapi::*;
//use render::i;

use maud::*;
use serde::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum QueryImage {
    Cropped100,
    Cropped300,
    Scaled300,
    Original,
}

#[derive(Clone, Debug)]
pub struct LinkConfig {
    pub http_root: String,
    pub static_root: String,
    pub resource_root: String,
    pub file_root: String,
    pub thumbnail_root: String,
    pub cache_bust: String,
}

/// Extend LinkConfig to have additional functionality
impl LinkConfig {
    pub fn image(&self, hash: &str, query: QueryImage) -> String {
        match query {
            QueryImage::Cropped100 => format!("{}/{}_s100.jpg", self.thumbnail_root, hash),
            QueryImage::Cropped300 => format!("{}/{}_s300.jpg", self.thumbnail_root, hash),
            QueryImage::Scaled300 => format!("{}/{}_300.jpg", self.thumbnail_root, hash),
            QueryImage::Original => format!("{}/{}", self.file_root, hash),
        }
    }

    // pub fn user(&self, user: &User) -> String {
    //     format!("{}/user/{}", self.http_root, user.username)
    // }
    pub fn user_unsafe(&self, username: &str) -> String {
        format!("{}/user/{}", self.http_root, username)
    }

    pub fn image_default(&self, hash: &str) -> String {
        self.image(hash, QueryImage::Original)
    }

    // pub fn votewidget(&self, content: &Content) -> String {
    //     format!("{}/widget/votes/{}", self.http_root, i(&content.id))
    // }
    pub fn votewidget_unsafe(&self, id: i64) -> String {
        format!("{}/widget/votes/{}", self.http_root, id)
    }

    // pub fn qr_generator(&self, content: &Content) -> String {
    //     format!("{}/widget/qr/{}", self.http_root, opt_s!(content.hash))
    // }
    pub fn qr_generator_unsafe(&self, hash: &str) -> String {
        format!("{}/widget/qr/{}", self.http_root, hash)
    }

    // pub fn forum_category(&self, category: &Content) -> String {
    //     self.forum_category_unsafe(opt_s!(category.hash))
    // }

    /// Create a category link using the current link system, which only uses the hash AVOID AS MUCH AS POSSIBLE!
    /// The implementation of the links may change!
    pub fn forum_category_unsafe(&self, hash: &str) -> String {
        format!("{}/forum/category/{}", self.http_root, hash)
    }

    // pub fn forum_thread(&self, thread: &Content) -> String {
    //     format!("{}/forum/thread/{}", self.http_root, opt_s!(thread.hash))
    // }
    pub fn forum_thread_unsafe(&self, hash: &str) -> String {
        format!("{}/forum/thread/{}", self.http_root, hash)
    }

    // pub fn forum_post_hash(post: &Message) -> String {
    //     format!("#post_{}", post.id.unwrap_or_default())
    // }

    // pub fn forum_post(&self, post: &Message, thread: &Content) -> String {
    //     format!(
    //         "{}/forum/thread/{}/{}{}",
    //         self.http_root,
    //         opt_s!(thread.hash),
    //         post.id.unwrap_or_default(),
    //         Self::forum_post_hash(post)
    //     )
    // }

    pub fn forum_post_unsafe(&self, pid: i64, thash: &str) -> String {
        format!(
            "{}/forum/thread/{}/{}#post_{}",
            self.http_root, thash, pid, pid,
        )
    }

    pub fn search_category(&self, category: i64) -> String {
        format!("{}/search?category={}", self.http_root, category)
    }

    // ---------------------
    //    FRAGMENTS
    // ---------------------
    pub fn style(&self, link: &str) -> Markup {
        html! {
            link rel="stylesheet" href={(self.static_root) (link) "?" (self.cache_bust) };
        }
    }

    pub fn script(&self, link: &str) -> Markup {
        html! {
            script src={(self.static_root) (link) "?" (self.cache_bust) } defer { }
        }
    }

    // Produce some metadata for the header that any page can use (even widgets)
    pub fn basic_meta(&self) -> Markup {
        html! {
            //Can I have comments in html markup?
            meta charset="UTF-8";
            meta name="rating" content="general";
            meta name="viewport" content="width=device-width";
            //[] is for optional, {} is for concatenate values
            link rel="icon" type="image/svg+xml" sizes="any" href={(self.resource_root) "/favicon.svg"};
        }
    }
}
