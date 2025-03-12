pub static COMMONCONTENT: &str =
    " deleted = 0 AND id IN (SELECT contentId FROM content_permissions WHERE read=1 AND userId=0) ";

pub fn query_childcount(idname: &str) -> String {
    return format!("SELECT COUNT(*) FROM content WHERE parentId = {}", idname);
}

pub fn query_vmap(count: usize) -> String {
    (0..count).map(|_| "?").collect::<Vec<_>>().join(",")
}
