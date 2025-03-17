use std::collections::HashMap;
use std::time::Instant;

use crate::response::*;

pub static BASICCONTENTFIELDS: &str = "c.id,c.hash,c.name,c.text";

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
