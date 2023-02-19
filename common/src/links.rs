use super::*;
use contentapi::*;
use contentapi::forms::*;
use render::i;

/// Extend LinkConfig to have additional functionality
impl LinkConfig {

    pub fn image(&self, hash: &str, query: &QueryImage) -> String 
    {
        match serde_urlencoded::to_string(&query) {
            Ok(querystring) => format!("{}/{}?{}", self.file_root, hash, querystring),
            Err(error) => {
                println!("Serde_qs failed? Not printing link for {}. Error: {}", hash, error);
                format!("#ERRORFOR-{}",hash)
            }
        }
    }

    pub fn user(&self, user: &User) -> String {
        format!("{}/user/{}", self.http_root, user.username)
    }

    pub fn image_default(&self, hash: &str) -> String { 
        self.image(hash, &QueryImage::default())
    }

    //pub fn page(&self, page: &Content) -> String {
    //    format!("{}/page/{}", self.http_root, opt_s!(page.hash))
    //}

    pub fn activity(&self) -> String {
        format!("{}/activity", self.http_root)
    }

    pub fn imagebrowser(&self) -> String {
        format!("{}/widget/imagebrowser", self.http_root)
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

    pub fn forum_post_hash(post: &Message) -> String {
        format!("#post_{}", post.id.unwrap_or_default())
    }

    pub fn forum_post(&self, post: &Message, thread: &Content) -> String {
        format!("{}/forum/thread/{}/{}{}", self.http_root, opt_s!(thread.hash), post.id.unwrap_or_default(), Self::forum_post_hash(post))
    }


    pub fn forum_thread_editor_new(&self, category: &Content) -> String {
        format!("{}/forum/edit/thread?category={}", self.http_root, opt_s!(category.hash))
    }

    pub fn forum_thread_editor_edit(&self, thread: &Content) -> String {
        format!("{}/forum/edit/thread?thread={}", self.http_root, opt_s!(thread.hash))
    }

    pub fn forum_thread_delete(&self, thread: &Content) -> String {
        format!("{}/forum/delete/thread/{}", self.http_root, i(&thread.id))
    }

    /// Get the link to the post editor for a brand new post. You HAVE to specify which thread you're posting on, but
    /// you can also optionally specify which post you're replying to.
    pub fn forum_post_editor_new(&self, thread: &Content, reply_to: Option<&Message>) -> String {
        format!("{}/forum/edit/post?thread={}{}", self.http_root, opt_s!(thread.hash),
            if let Some(reply) = reply_to {
                format!("&reply={}", i(&reply.id))
            } else {
                String::from("")
            })
    }

    /// Get the link to the post editor to edit the given message. You don't need extra data in this case, since 
    /// the message to edit has all the info you need
    pub fn forum_post_editor_edit(&self, post: &Message) -> String {
        format!("{}/forum/edit/post?post={}", self.http_root, i(&post.id))
    }

    /// Get the link to delete a post. You'll need to POST to this to delete
    pub fn forum_post_delete(&self, post: &Message) -> String {
        format!("{}/forum/delete/post/{}", self.http_root, i(&post.id))
    }


    pub fn page_editor_new(&self, page_type: &str) -> String {
        format!("{}/page/edit?type={}", self.http_root, page_type)
    }

    pub fn page_editor_edit(&self, page: &Content) -> String {
        format!("{}/page/edit?page={}", self.http_root, opt_s!(page.hash))
    }

    pub fn page_delete(&self, page: &Content) -> String {
        format!("{}/page/delete/{}", self.http_root, i(&page.id))
    }


    pub fn search_category(&self, category: i64) -> String {
        format!("{}/search?category={}", self.http_root, category)
    }

}

impl MainLayoutData 
{
    /// Get a plain path (no query) pointing to this current request. This SHOULD work anywhere...
    /// but how often do you REALLY want this one?
    pub fn current(&self) -> String {
        format!("{}{}", self.links.http_root, self.current_path)
    }
}

