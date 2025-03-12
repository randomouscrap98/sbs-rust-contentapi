use chrono::{DateTime, Utc};

use super::*;
use crate::constants::*;
use crate::response::*;

pub static CATEGORYFIELDS2: &str = "id,hash,name,COALESCE(description,'') AS description,COALESCE(literalType,'') AS literalType,contentType";
pub static THREADFIELDS2: &str =
    "id,hash,name,COALESCE(literalType,''),contentType,parentId,createDate,createUserId";
pub static COMMONCONTENT: &str =
    " deleted = 0 AND id IN (SELECT contentId FROM content_permissions WHERE read=1 AND userId=0) ";
pub static VALUESELECT: &str = "SELECT key,value FROM content_values WHERE contentId=?";

pub fn select_childcount(idname: &str) -> String {
    return format!("SELECT COUNT(*) FROM content WHERE parentId = {}", idname);
}
pub fn select_postcount(idname: &str) -> String {
    return format!("SELECT COUNT(*) FROM messages WHERE contentId = {}", idname);
}
pub fn select_maxpost(idname: &str) -> String {
    return format!(
        "SELECT COALESCE(MAX(id),0) FROM messages WHERE contentId = {}",
        idname
    );
}

pub fn params_list(count: usize) -> String {
    (0..count).map(|_| "?").collect::<Vec<_>>().join(",")
}

pub fn gather_values(
    vstmt: &mut rusqlite::Statement,
    id: i64,
) -> Result<HashMap<String, String>, rusqlite::Error> {
    let value_iter = vstmt.query_map([&id], |row| {
        Ok((row.get::<usize, String>(0)?, row.get::<usize, String>(1)?))
    })?;
    let mut result: HashMap<String, String> = HashMap::new();
    for kv in value_iter {
        let rkv = kv?;
        result.insert(rkv.0, rkv.1);
    }
    Ok(result)
}

#[derive(Debug, Default, Clone)]
pub struct QueryLimit {
    pub limit: Option<i32>,
    pub skip: Option<i32>,
}

impl QueryLimit {
    pub fn mod_query(&self, query: &mut String, params: &mut Vec<Box<dyn rusqlite::types::ToSql>>) {
        if let Some(limit) = self.limit {
            query.push_str(" LIMIT ?");
            params.push(Box::new(limit));
        }
        if let Some(skip) = self.skip {
            query.push_str(" OFFSET ?");
            params.push(Box::new(skip));
        }
    }
}

#[derive(Debug, Clone)]
pub enum IdOrHash {
    Id(i64),
    Hash(String),
}

impl IdOrHash {
    pub fn mod_query(&self, query: &mut String) -> Box<dyn rusqlite::types::ToSql> {
        match self {
            IdOrHash::Id(fcid) => {
                query.push_str(" AND id = ?");
                Box::new(*fcid)
            }
            IdOrHash::Hash(hash) => {
                query.push_str(" AND hash = ?");
                Box::new(hash.clone())
            }
        }
    }
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
        params_list(ids.len())
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

pub fn get_categories(
    ctx: &PageContext,
    cq: Option<IdOrHash>,
) -> Result<Vec<ForumCategory2>, Error> {
    let mut query = format!(
        "SELECT {},({}) AS thread_count FROM content c WHERE {} AND literalType IN ({})",
        CATEGORYFIELDS2,
        select_childcount("c.id"),
        COMMONCONTENT,
        params_list(FORUMCATEGORYTYPES.len())
    );
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    for t in FORUMCATEGORYTYPES.iter() {
        params.push(Box::new(*t));
    }
    if let Some(cq) = cq {
        params.push(cq.mod_query(&mut query));
    }
    let mut stmt = ctx.dbcon.prepare(&query)?;
    let category_iter = stmt.query_map(
        params
            .iter()
            .map(|x| x.as_ref())
            .collect::<Vec<_>>()
            .as_slice(),
        |row| {
            Ok(ForumCategory2 {
                id: row.get(0)?,
                hash: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                literal_type: row.get(4)?,
                content_type: row.get(5)?,
                threads_count: row.get(6)?,
            })
        },
    )?;

    #[cfg(feature = "querydump")]
    println!("Query: {:?}", &query);

    Ok(category_iter.collect::<Result<Vec<ForumCategory2>, rusqlite::Error>>()?)
}

#[derive(Clone, Debug)]
pub struct ForumThread2 {
    pub id: i64,
    pub hash: String,
    pub name: String,
    pub literal_type: String,
    pub content_type: i64,
    pub parent_id: i64,
    pub create_date: DateTime<Utc>,
    pub create_user_id: i64,
    pub posts_count: i32,
    pub max_post_id: i64,
    pub values: HashMap<String, String>,
}

pub fn get_threads(
    ctx: &PageContext,
    tquery: Option<IdOrHash>,
    category_id: Option<i64>,
    limits: QueryLimit,
) -> Result<Vec<ForumThread2>, Error> {
    let mut query = format!(
        "SELECT {},({}) AS post_count,({}) AS max_post FROM content t WHERE {} AND contentType = ? AND literalType IN ({})",
        THREADFIELDS2,
        select_postcount("t.id"),
        select_maxpost("t.id"),
        COMMONCONTENT,
        params_list(THREADTYPES.len())
    );
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(ContentType::PAGE)];
    for t in THREADTYPES.iter() {
        params.push(Box::new(*t));
    }
    if let Some(tq) = tquery {
        params.push(tq.mod_query(&mut query));
    }
    if let Some(cid) = category_id {
        query.push_str(" AND parentId = ?");
        params.push(Box::new(cid));
    }
    query.push_str(" ORDER BY max_post DESC");
    limits.mod_query(&mut query, &mut params);
    let mut stmt = ctx.dbcon.prepare(&query)?;
    let mut vstmt = ctx.dbcon.prepare(VALUESELECT)?;
    let thread_iter = stmt.query_map(
        params
            .iter()
            .map(|x| x.as_ref())
            .collect::<Vec<_>>()
            .as_slice(),
        |row| {
            let id = row.get(0)?;
            Ok(ForumThread2 {
                id,
                hash: row.get(1)?,
                name: row.get(2)?,
                literal_type: row.get(3)?,
                content_type: row.get(4)?,
                parent_id: row.get(5)?,
                create_date: row.get(6)?,
                create_user_id: row.get(7)?,
                posts_count: row.get(8)?,
                max_post_id: row.get(9)?,
                values: gather_values(&mut vstmt, id)?,
            })
        },
    )?;

    #[cfg(feature = "querydump")]
    println!("Query: {:?}", &query);

    Ok(thread_iter.collect::<Result<Vec<ForumThread2>, rusqlite::Error>>()?)
}

// Get engagement for a particular content
pub fn get_engagements(ctx: &PageContext, content: i64) -> Result<HashMap<String, i32>, Error> {
    // WARN: you can get engagements for content that is private! You also get engagements
    // for deleted users/etc!
    let query = format!("SELECT `type`,engagement,count(*) FROM content_engagement WHERE contentId = ? GROUP BY `type`,engagement");
    let mut stmt = ctx.dbcon.prepare(&query)?;
    let engagement_iter = stmt.query_map([&content], |row| {
        let mut typ: String = row.get(0)?;
        let eng: String = row.get(1)?;
        typ.push_str(&eng);
        let count: i32 = row.get(2)?;
        Ok((typ, count))
    })?;

    #[cfg(feature = "querydump")]
    println!("Query: {:?}", &query);

    let mut result = HashMap::<String, i32>::new();

    for engagement in engagement_iter {
        let reng = engagement?;
        result.insert(reng.0, reng.1);
    }

    Ok(result)
}
