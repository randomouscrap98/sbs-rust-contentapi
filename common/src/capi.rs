use chrono::{DateTime, Utc};

use super::*;
use crate::constants::*;
use crate::response::*;

pub static CATEGORYFIELDS2: &str = "id,hash,name,COALESCE(description,'') AS description,COALESCE(literalType,'') AS literalType,contentType";
pub static COMMONCONTENT: &str =
    " deleted = 0 AND id IN (SELECT contentId FROM content_permissions WHERE read=1 AND userId=0) ";

pub fn query_childcount(idname: &str) -> String {
    return format!("SELECT COUNT(*) FROM content WHERE parentId = {}", idname);
}

pub fn query_vmap(count: usize) -> String {
    (0..count).map(|_| "?").collect::<Vec<_>>().join(",")
}

#[derive(Clone, Debug)]
pub struct User2 {
    pub id: i64,
    pub user_type: i8,
    pub username: String,
    pub avatar: String,
    pub special: Option<String>,
    pub admin: bool,
    pub create_date: DateTime<Utc>,
    //pub groups: Vec<i64>,
}

pub fn get_users(ctx: &PageContext, ids: Vec<i64>) -> Result<Vec<User2>, Error> {
    let query = format!("SELECT id,`type`,username,avatar,special,super,createDate FROM users WHERE deleted=0 AND id IN ({})",
        query_vmap(ids.len())
        );
    let mut params: Vec<&dyn rusqlite::types::ToSql> = vec![];
    for t in ids.iter() {
        params.push(t);
    }

    let mut stmt = ctx.dbcon.prepare(&query)?;
    let user_iter = stmt.query_map(params.as_slice(), |row| {
        Ok(User2 {
            id: row.get(0)?,
            user_type: row.get(1)?,
            username: row.get(2)?,
            avatar: row.get(3)?,
            special: row.get(4)?,
            admin: row.get(5)?,
            create_date: row.get(6)?,
        })
    })?;

    #[cfg(feature = "querydump")]
    println!("Query: {:?}", &query);

    Ok(user_iter.collect::<Result<Vec<User2>, rusqlite::Error>>()?)
}

//Structs JUST for building data for the forum templates (so no need to be public)
#[derive(Clone, Debug)]
pub struct ForumCategory2 {
    pub id: i64,
    pub hash: String,
    pub name: String,
    pub description: String,
    pub literal_type: String,
    pub content_type: i64,
    pub threads_count: i32,
}

pub fn get_categories(ctx: &PageContext, fcid: Option<i64>) -> Result<Vec<ForumCategory2>, Error> {
    let mut query = format!(
        "SELECT {},({}) AS thread_count FROM content c WHERE {} AND literalType IN ({})",
        CATEGORYFIELDS2,
        query_childcount("c.id"),
        COMMONCONTENT,
        query_vmap(FORUMCATEGORYTYPES.len())
    );
    let mut params: Vec<&dyn rusqlite::types::ToSql> = vec![];
    for t in FORUMCATEGORYTYPES.iter() {
        params.push(t);
    }
    let tfcid: i64;
    if let Some(fcid) = fcid {
        query.push_str(" AND id = ?");
        tfcid = fcid;
        params.push(&tfcid);
    }
    let mut stmt = ctx.dbcon.prepare(&query)?;
    let category_iter = stmt.query_map(params.as_slice(), |row| {
        Ok(ForumCategory2 {
            id: row.get(0)?,
            hash: row.get(1)?,
            name: row.get(2)?,
            description: row.get(3)?,
            literal_type: row.get(4)?,
            content_type: row.get(5)?,
            threads_count: row.get(6)?,
        })
    })?;

    #[cfg(feature = "querydump")]
    println!("Query: {:?}", &query);

    Ok(category_iter.collect::<Result<Vec<ForumCategory2>, rusqlite::Error>>()?)
}
