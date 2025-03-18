#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::time::Instant;

use super::*;
use crate::pages::context::*;
use crate::response::*;

#[macro_export]
macro_rules! string_const {
    ($name:ident => {
        $(($item:ident:$val:literal)),*$(,)?
    }) => {
        #[allow(dead_code)] //man, idk if i'll use ALL of them but I WANT them
        pub enum $name { }

        impl $name {
        $(
            pub const $item: &str = $val;
        )*
        }
    };
}

#[macro_export]
macro_rules! byte_enum {
    ($name:ident => {
        $(($item:ident : $val:literal)),*$(,)?
    }) => {
        #[allow(dead_code)] //man, idk if i'll use ALL of them but I WANT them
        pub enum $name { }

        impl $name {
        $(
            pub const $item: i8 = $val;
        )*
        }
    };
}

pub const UPVOTE: &str = "+";
pub const DOWNVOTE: &str = "-";
pub const VOTETYPE: &str = "vote";
pub const UPVOTESTR: &str = "vote+";
pub const DOWNVOTESTR: &str = "vote-";

pub const CATEGORYPREFIX: &str = "tag:";
//pub const ANYSYSTEM: &str = "any";
//pub const PTCSYSTEM: &str = "ptc";

byte_enum! { ContentType => {
    (PAGE:1i8),
    (MODULE:2i8),
    (FILE:3i8),
    (USERPAGE:4i8),
    (SYSTEM:5i8)
}}

byte_enum! { UserType => {
    (USER:1i8),
    (GROUP:2i8)
}}

byte_enum! { UserRelationType => {
    (INGROUP:1i8),
    (ASSIGNCONTENT:2i8)
}}

string_const! { SBSValue => {
    (DOWNLOADKEY:"dlkey"),
    (VERSION:"version"),
    (SIZE:"size"),
    (SYSTEMS:"systems"),
    (IMAGES:"images"),
    (FORCONTENT:"forcontent"),
    (MARKUP:"markup"),
    (DOCPATH:"docpath")
}}

string_const! { SBSPageType => {
    (PROGRAM:"program"),
    (RESOURCE:"resource"),
    (CATEGORY:"category"),
    (FORUMCATEGORY:"forumcategory"),
    (FORUMTHREAD:"forumthread"),
    (DIRECTMESSAGES:"directmessages"),
    (DIRECTMESSAGE:"directmessage"),
    (ALERT:"alert"),
    (FRONTPAGE:"frontpage"),
    (DOCSCUSTOM: "docscustom"),
    (SUBMISSIONS:"submissions"),
    (PTCFILES:"ptcfiles"),
    (DOCPARENT:"docparent"),
    (DOCUMENTATION:"documentation")
}}

pub const THREADTYPES: &[&str] = &[
    SBSPageType::FORUMTHREAD,
    SBSPageType::PROGRAM,
    SBSPageType::RESOURCE,
    SBSPageType::DIRECTMESSAGE,
    SBSPageType::DOCUMENTATION,
];

pub static BASICCONTENTFIELDS: &str = "c.id,c.hash,c.name,c.text";
pub static USER2FIELDS: &str = "id,`type`,username,avatar,special,super,createDate";
pub static BROWSEFIELDS: &str =
     "c.id,c.hash,c.name,COALESCE(c.description,''),COALESCE(c.literalType,''),c.createDate,c.createUserId";

pub static VALUESELECT: &str = "SELECT `key`,`value` FROM content_values WHERE contentId=?";
pub static KEYWORDSELECT: &str = "SELECT `value` FROM content_keywords WHERE contentId=?";
pub static COMMONCONTENT: &str =
    " deleted = 0 AND id IN (SELECT contentId FROM content_permissions WHERE read=1 AND userId=0) ";
pub static COMMONUSER: &str =
    " deleted = 0 AND `type`= 1 AND (registrationKey IS NULL OR registrationKey = '') ";

pub fn join_common_content(idname: &str) -> String {
    return format!(
        "JOIN content_permissions _cp_ ON _cp_.contentId = {} AND _cp_.read=1 AND _cp_.userId=0",
        idname
    );
}

pub fn easy_query<T, F>(
    stmt: (&mut rusqlite::Statement, &str),
    params: &[&dyn rusqlite::ToSql],
    rowmap: F,
) -> Result<Vec<T>, Error>
where
    F: FnMut(&rusqlite::Row<'_>) -> Result<T, rusqlite::Error>,
{
    #[cfg(feature = "querydump")]
    let start = Instant::now();
    let (stmt, query) = stmt;
    let row_iter = stmt.query_map(params, rowmap)?;
    let result = row_iter.collect::<Result<Vec<T>, rusqlite::Error>>()?;

    #[cfg(feature = "querydump")]
    {
        let duration = start.elapsed();
        println!("Query ({} ms): {:?}", duration.as_millis(), query);
    }
    Ok(result)
}

#[macro_export]
macro_rules! box_to_ref {
    ($params:expr) => {{
        $params
            .iter()
            .map(|x| x.as_ref())
            .collect::<Vec<_>>()
            .as_slice()
    }};
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
pub fn maybe_gather_values(
    vstmt: &mut Option<rusqlite::Statement>,
    id: i64,
) -> Result<HashMap<String, String>, rusqlite::Error> {
    if let Some(vstmt) = vstmt.as_mut() {
        gather_values(vstmt, id)
    } else {
        Ok(HashMap::default())
    }
}

pub fn gather_keywords(
    kstmt: &mut rusqlite::Statement,
    id: i64,
) -> Result<Vec<String>, rusqlite::Error> {
    let key_iter = kstmt.query_map([&id], |row| Ok(row.get::<usize, String>(0)?))?;
    Ok(key_iter.collect::<Result<Vec<String>, rusqlite::Error>>()?)
}

#[macro_export]
macro_rules! get_value_safe {
    ($values:expr, $key:expr, $T:ty) => {{
        if let Some(v) = $values.get($key) {
            match serde_json::from_str::<$T>(v) {
                Ok(v) => Some(v),
                Err(error) => {
                    println!("Couldn't parse '{}' from content values: {}", $key, error);
                    None
                }
            }
        } else {
            None
        }
    }};
}

pub fn get_systems2(values: &HashMap<String, String>) -> Vec<String> {
    if let Some(systems) = values.get(SBSValue::SYSTEMS) {
        match serde_json::from_str(systems) {
            Ok(v) => v,
            Err(e) => {
                println!("Can't parse systems: {}", e);
                Vec::new()
            }
        }
    } else {
        Vec::new()
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

pub fn user_or_default2(user: Option<&User2>) -> User2 {
    if let Some(u) = user {
        u.clone()
    } else {
        User2 {
            id: 0,
            username: String::from("???"),
            avatar: String::from("0"),
            user_type: UserType::USER,
            admin: false,
            special: None,
            create_date: chrono::Utc::now(),
        }
    }
}

pub fn map_users2(users: Vec<User2>) -> HashMap<i64, User2> {
    users
        .into_iter()
        .map(|u| (u.id, u))
        .collect::<HashMap<i64, User2>>()
}

pub fn gather_users(
    stmt: (&mut rusqlite::Statement, &str),
    params: &[&dyn rusqlite::ToSql],
) -> Result<Vec<User2>, Error> {
    easy_query(stmt, params, |row| {
        Ok(User2 {
            id: row.get(0)?,
            user_type: row.get(1)?,
            username: row.get(2)?,
            avatar: row.get(3)?,
            special: row.get(4)?,
            admin: row.get(5)?,
            create_date: row.get(6)?,
        })
    })
}

pub fn get_users(ctx: &PageContext, ids: Vec<i64>) -> Result<Vec<User2>, Error> {
    let query = format!(
        "SELECT {} FROM users WHERE {} AND id IN ({})",
        USER2FIELDS,
        COMMONUSER,
        params_list(ids.len()),
    );
    let mut params: Vec<&dyn rusqlite::types::ToSql> = vec![];
    for t in ids.iter() {
        params.push(t);
    }
    let mut stmt = ctx.dbcon.prepare(&query)?;

    gather_users((&mut stmt, &query), params.as_slice())
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
/// For SPECIFICALLY the old id system, used for ftid, fcid, fpid, etc
pub enum OldIdOrHash {
    Id(i64),
    Hash(String),
}

impl OldIdOrHash {
    pub fn mod_query(
        &self,
        table_name: &str,
        key_name: &'static str,
        query: &mut String,
        params: &mut Vec<Box<dyn rusqlite::ToSql>>,
    ) {
        match self {
            OldIdOrHash::Id(id) => {
                // I'm NOT SURE why exists is faster, since the inner query should only return one
                // value but the outer query has loads of values... hmmm
                query.push_str(&format!(" AND EXISTS (SELECT id FROM content_values WHERE `key`=? AND `value`=? AND contentId={}.id)", table_name));
                params.push(Box::new(key_name));
                params.push(Box::new(*id));
            }
            OldIdOrHash::Hash(hash) => {
                query.push_str(&format!(" AND {}.hash = ?", table_name));
                params.push(Box::new(hash.clone()));
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct BasicContent {
    pub id: i64,
    pub hash: String,
    pub name: String,
    pub text: String,
}

pub fn gather_basiccontent(
    stmt: (&mut rusqlite::Statement, &str),
    params: &[&dyn rusqlite::ToSql],
) -> Result<Vec<BasicContent>, Error> {
    easy_query(stmt, params, |row| {
        let id: i64 = row.get(0)?;
        Ok(BasicContent {
            id,
            hash: row.get(1)?,
            name: row.get(2)?,
            text: row.get(3)?,
        })
    })
}

#[derive(Debug, Clone)]
pub struct SubmissionCategory {
    pub id: i64,
    pub name: String,
    pub hash: String,
    pub forcontent: String,
}

pub fn get_submission_categories(
    ctx: &PageContext,
    ids: Option<Vec<i64>>,
) -> Result<Vec<SubmissionCategory>, Error> {
    let mut query = format!(
        r##"SELECT c.id,c.name,c.hash,v.`value`
        FROM content c JOIN content_values v ON c.id=v.contentId {}
        WHERE v.`key`=? AND c.contentType = ? AND c.literalType = ?"##,
        //(SELECT `value` FROM content_values WHERE contentId=c.id AND `key`=?) FROM content AS c WHERE {} AND contentType = ? AND literalType = ?",
        join_common_content("c.id"),
    );

    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![
        Box::new(SBSValue::FORCONTENT),
        Box::new(ContentType::SYSTEM),
        Box::new(SBSPageType::CATEGORY),
    ];

    if let Some(ids) = ids {
        // Bypass if no ids
        if ids.len() == 0 {
            #[cfg(feature = "querydump")]
            println!("Skipping category query: no ids");
            return Ok(Vec::new());
        }
        query.push_str(&format!(" AND c.id IN ({})", params_list(ids.len())));
        for id in ids {
            params.push(Box::new(id));
        }
    }

    let mut stmt = ctx.dbcon.prepare(&query)?;
    return easy_query((&mut stmt, &query), box_to_ref!(params), |row| {
        let cval: Option<String> = row.get(3)?;
        Ok(SubmissionCategory {
            id: row.get(0)?,
            name: row.get(1)?,
            hash: row.get(2)?,
            forcontent: if let Some(cval) = cval {
                serde_json::from_str(&cval).unwrap_or_default()
            } else {
                String::new()
            },
        })
    });
}

#[derive(Clone, Debug)]
pub struct BrowseContent {
    pub id: i64,
    pub hash: String,
    pub name: String,
    pub description: String,
    pub literal_type: String,
    pub create_date: DateTime<Utc>,
    pub create_user_id: i64,
    pub values: HashMap<String, String>,
}

pub fn gather_browsecontent(
    stmt: (&mut rusqlite::Statement, &str),
    vstmt: &mut Option<rusqlite::Statement>,
    params: &[&dyn rusqlite::ToSql],
) -> Result<Vec<BrowseContent>, Error> {
    easy_query(stmt, params, |row| {
        let id: i64 = row.get(0)?;
        Ok(BrowseContent {
            id,
            hash: row.get(1)?,
            name: row.get(2)?,
            description: row.get(3)?,
            literal_type: row.get(4)?,
            create_date: row.get(5)?,
            create_user_id: row.get(6)?,
            values: maybe_gather_values(vstmt, id)?,
        })
    })
}

pub fn get_browse(
    ctx: &PageContext,
    search: &PageSearch,
    limits: QueryLimit,
) -> Result<Vec<BrowseContent>, Error> {
    let mut query = format!(
        r##"SELECT {},(SELECT COUNT(*) FROM content_engagement e WHERE contentId=c.id AND e.`type`=? AND e.`engagement`=?) AS upvotes
        FROM content c JOIN content p ON c.parentId=p.id {}
        WHERE c.contentType=? AND p.contentType = ? AND p.literalType = ?"##,
        BROWSEFIELDS,
        join_common_content("c.id"),
    );
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![
        Box::new(VOTETYPE),
        Box::new(UPVOTE),
        Box::new(ContentType::PAGE),
        Box::new(ContentType::SYSTEM),
        Box::new(SBSPageType::SUBMISSIONS),
    ];
    if let Some(stext) = &search.search {
        params.push(Box::new(format!("%{}%", stext)));
        params.push(Box::new(format!("%{}%", stext)));
        query.push_str(" AND (c.name LIKE ? OR c.id IN (SELECT contentId FROM content_keywords WHERE `value` LIKE ?))");
    }

    if let Some(category) = search.category {
        if category != 0 {
            params.push(Box::new(format!("{}{}", CATEGORYPREFIX, category)));
            query.push_str(" AND c.id IN (SELECT contentId FROM content_values WHERE `key` = ?)");
        }
    }

    if let Some(user_id) = search.user_id {
        if user_id != 0 {
            params.push(Box::new(user_id));
            query.push_str(" AND c.createUserId = ?");
        }
    }

    if let Some(subtype) = &search.subtype {
        if !subtype.is_empty() {
            params.push(Box::new(subtype.clone()));
            query.push_str(" AND c.literalType = ?");
            if subtype == SBSPageType::PROGRAM {
                //MUST have a key unless the user specifies otherwise
                if !search.removed {
                    params.push(Box::new(SBSValue::DOWNLOADKEY));
                    params.push(Box::new(SBSValue::SYSTEMS));
                    params.push(Box::new(format!("%{}%", PTCSYSTEM)));
                    query.push_str(
                        " AND c.id IN (SELECT contentId FROM content_values WHERE `key`= ? OR (`key` = ? AND `value` LIKE ?))" //(!valuekeyin(@dlkeylist) or !valuelike(@systemkey, @ptcsystem))",
                    );
                }
                if search.system != ANYSYSTEM {
                    params.push(Box::new(SBSValue::SYSTEMS));
                    params.push(Box::new(format!("%{}%", search.system)));
                    query.push_str(" AND c.id IN (SELECT contentId FROM content_values WHERE `key`=? AND `value` LIKE ?)");
                }
            }
        }
    }

    query.push_str(" GROUP BY c.id");

    if search.order == "id" {
        query.push_str(" ORDER BY c.id");
    } else if search.order == "id_desc" {
        query.push_str(" ORDER BY c.id DESC");
    } else if search.order == "upvotes" {
        query.push_str(" ORDER BY upvotes DESC");
    } else if search.order == "name" {
        query.push_str(" ORDER BY name");
    } else if search.order == "name_desc" {
        query.push_str(" ORDER BY name DESC");
    } else {
        return Err(Error::User(format!(
            "Unknown search order: {}",
            search.order
        )));
    }

    limits.mod_query(&mut query, &mut params);

    let mut stmt = ctx.dbcon.prepare(&query)?;
    let vstmt = ctx.dbcon.prepare(VALUESELECT)?;

    gather_browsecontent((&mut stmt, &query), &mut Some(vstmt), box_to_ref!(params))
}
