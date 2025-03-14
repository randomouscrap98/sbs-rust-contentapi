use super::*;
use contentapi::*;
use render::i;

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

    pub fn user(&self, user: &User) -> String {
        format!("{}/user/{}", self.http_root, user.username)
    }
    pub fn user_unsafe(&self, username: &str) -> String {
        format!("{}/user/{}", self.http_root, username)
    }

    pub fn image_default(&self, hash: &str) -> String {
        self.image(hash, QueryImage::Original)
    }

    pub fn votewidget(&self, content: &Content) -> String {
        format!("{}/widget/votes/{}", self.http_root, i(&content.id))
    }

    pub fn qr_generator(&self, content: &Content) -> String {
        format!("{}/widget/qr/{}", self.http_root, opt_s!(content.hash))
    }

    pub fn forum_category(&self, category: &Content) -> String {
        self.forum_category_unsafe(opt_s!(category.hash))
    }

    /// Create a category link using the current link system, which only uses the hash AVOID AS MUCH AS POSSIBLE!
    /// The implementation of the links may change!
    pub fn forum_category_unsafe(&self, hash: &str) -> String {
        format!("{}/forum/category/{}", self.http_root, hash)
    }

    pub fn forum_thread(&self, thread: &Content) -> String {
        format!("{}/forum/thread/{}", self.http_root, opt_s!(thread.hash))
    }
    pub fn forum_thread_unsafe(&self, hash: &str) -> String {
        format!("{}/forum/thread/{}", self.http_root, hash)
    }

    pub fn forum_post_hash(post: &Message) -> String {
        format!("#post_{}", post.id.unwrap_or_default())
    }

    pub fn forum_post(&self, post: &Message, thread: &Content) -> String {
        format!(
            "{}/forum/thread/{}/{}{}",
            self.http_root,
            opt_s!(thread.hash),
            post.id.unwrap_or_default(),
            Self::forum_post_hash(post)
        )
    }

    pub fn forum_post_unsafe(&self, pid: i64, thash: &str) -> String {
        format!(
            "{}/forum/thread/{}/{}{}",
            self.http_root,
            thash,
            pid,
            format!("#post_{}", pid),
        )
    }

    pub fn search_category(&self, category: i64) -> String {
        format!("{}/search?category={}", self.http_root, category)
    }
}

impl MainLayoutData {
    /// Get a plain path (no query) pointing to this current request. This SHOULD work anywhere...
    /// but how often do you REALLY want this one?
    pub fn current(&self) -> String {
        format!("{}{}", self.links.http_root, self.current_path)
    }
}
