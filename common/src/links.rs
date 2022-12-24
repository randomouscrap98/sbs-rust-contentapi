use super::*;
use contentapi::*;
use contentapi::forms::*;


/// Extend LinkConfig to have additional functionality
impl LinkConfig {

    pub fn image(&self, hash: &str, query: &QueryImage) -> String {
        //let query = contentapi::forms::QueryImage { 
        //    size : if size > 0 { Some(size as i64) } else { None },
        //    crop : if crop { Some(crop) } else { None }
        //};
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

    pub fn page(&self, page: &Content) -> String {
        format!("{}/page/{}", self.http_root, opt_s!(page.hash))
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

}

impl MainLayoutData 
{
    /// Get a plain path (no query) pointing to this current request. This SHOULD work anywhere...
    /// but how often do you REALLY want this one?
    pub fn current(&self, data: &MainLayoutData) -> String {
        format!("{}{}", data.links.http_root, data.current_path)
    }
}

