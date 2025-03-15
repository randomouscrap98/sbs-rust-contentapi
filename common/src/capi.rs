use chrono::{DateTime, Utc};
use std::time::Instant;

use super::*;
use crate::constants::*;
use crate::response::*;

pub static BASICCONTENTFIELDS: &str = "id,hash,name,text";
pub static BROWSEFIELDS: &str =
    "id,hash,name,COALESCE(description,''),COALESCE(literalType,''),createDate,createUserId";
pub static USER2FIELDS: &str = "id,`type`,username,avatar,special,super,createDate";
pub static THREADFIELDS2: &str =
    "id,hash,name,COALESCE(literalType,''),contentType,parentId,createDate,createUserId";
pub static COMMONCONTENT: &str =
    " deleted = 0 AND id IN (SELECT contentId FROM content_permissions WHERE read=1 AND userId=0) ";
pub static COMMONUSER: &str =
    " deleted = 0 AND `type`= 1 AND (registrationKey IS NULL OR registrationKey = '') ";
pub static VALUESELECT: &str = "SELECT `key`,`value` FROM content_values WHERE contentId=?";
pub static KEYWORDSELECT: &str = "SELECT `value` FROM content_keywords WHERE contentId=?";
//pub static SUBMISSIONPARENT: &str = "SELECT id FROM content WHERE contentId=?";

pub fn join_common_content(idname: &str) -> String {
    return format!(
        "JOIN content_permissions _cp_ ON _cp_.contentId = {} AND _cp_.read=1 AND _cp_.userId=0",
        idname
    );
}
// pub fn select_childcount(idname: &str) -> String {
//     return format!("SELECT COUNT(*) FROM content WHERE parentId = {}", idname);
// }
pub fn select_postcount(idname: &str) -> String {
    return format!("SELECT COUNT(*) FROM messages WHERE contentId = {}", idname);
}
pub fn select_maxpost(idname: &str) -> String {
    return format!(
        "SELECT COALESCE(MAX(id),0) FROM messages WHERE contentId = {}",
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
pub enum IdOrHash {
    Id(i64),
    Hash(String),
}

impl IdOrHash {
    pub fn mod_query(
        &self,
        table_name: &str,
        query: &mut String,
    ) -> Box<dyn rusqlite::types::ToSql> {
        match self {
            IdOrHash::Id(fcid) => {
                query.push_str(&format!(" AND {}.id = ?", table_name));
                Box::new(*fcid)
            }
            IdOrHash::Hash(hash) => {
                query.push_str(&format!(" AND {}.hash = ?", table_name));
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

// Find user by name
pub fn get_user_by_name(ctx: &PageContext, name: &str) -> Result<Option<User2>, Error> {
    let query = format!(
        "SELECT {} FROM users WHERE {} AND username = ?",
        USER2FIELDS, COMMONUSER,
    );
    let params: Vec<&dyn rusqlite::types::ToSql> = vec![&name];
    let mut stmt = ctx.dbcon.prepare(&query)?;

    let mut users = gather_users((&mut stmt, &query), params.as_slice())?;
    Ok(users.pop())
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

pub fn get_forum_categories(
    ctx: &PageContext,
    cq: Option<IdOrHash>,
) -> Result<Vec<ForumCategory2>, Error> {
    let mut query = format!(
        "SELECT {} FROM content c JOIN content t ON c.id=t.parentId {} WHERE c.literalType IN ({}) GROUP BY c.id",
        "c.id,c.hash,c.name,COALESCE(c.description,''),COALESCE(c.literalType,''),c.contentType,count(t.id)",
        join_common_content("c.id"),
        params_list(FORUMCATEGORYTYPES.len())
    );
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    for t in FORUMCATEGORYTYPES.iter() {
        params.push(Box::new(*t));
    }
    if let Some(cq) = cq {
        params.push(cq.mod_query("c", &mut query));
    }
    let mut stmt = ctx.dbcon.prepare(&query)?;
    easy_query((&mut stmt, &query), box_to_ref!(params), |row| {
        Ok(ForumCategory2 {
            id: row.get(0)?,
            hash: row.get(1)?,
            name: row.get(2)?,
            description: row.get(3)?,
            literal_type: row.get(4)?,
            content_type: row.get(5)?,
            threads_count: row.get(6)?,
        })
    })
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
        params.push(tq.mod_query("t", &mut query));
    }
    if let Some(cid) = category_id {
        query.push_str(" AND parentId = ?");
        params.push(Box::new(cid));
    }
    query.push_str(" ORDER BY max_post DESC");
    limits.mod_query(&mut query, &mut params);
    let mut stmt = ctx.dbcon.prepare(&query)?;
    let mut vstmt = ctx.dbcon.prepare(VALUESELECT)?;
    easy_query((&mut stmt, &query), box_to_ref!(params), |row| {
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
    })
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

// #[derive(Clone, Debug)]
// pub struct RawContent {
//     pub id: i64,
//     pub create_user_id: i64,
//     pub create_date: DateTime<Utc>,
//     pub content_type: i64,
//     pub literal_type: String,
//     pub meta: Option<String>, // Option because this is parsed as json later
//     pub description: String,
//     pub hash: String,
//     pub name: String,
//     pub text: String,
//     pub parent_id: i64,
// }

#[derive(Clone, Debug)]
pub struct BasicContent {
    pub id: i64,
    pub hash: String,
    pub name: String,
    pub text: String,
}

pub fn gather_basiccontent(
    stmt: &mut rusqlite::Statement,
    params: &[&dyn rusqlite::ToSql],
) -> Result<Vec<BasicContent>, Error> {
    let basic_iter = stmt.query_map(params, |row| {
        let id: i64 = row.get(0)?;
        Ok(BasicContent {
            id,
            hash: row.get(1)?,
            name: row.get(2)?,
            text: row.get(3)?,
        })
    })?;

    return Ok(basic_iter.collect::<Result<Vec<BasicContent>, rusqlite::Error>>()?);
}

// Get any system content with given literaltype
pub fn get_systempage(ctx: &PageContext, literal_type: String) -> Result<Vec<BasicContent>, Error> {
    let query = format!(
        "SELECT {} FROM content WHERE {} AND contentType = ? AND literalType = ?",
        BASICCONTENTFIELDS, COMMONCONTENT
    );
    let mut stmt = ctx.dbcon.prepare(&query)?;
    #[cfg(feature = "querydump")]
    println!("Query: {:?}", &query);

    gather_basiccontent(
        &mut stmt,
        rusqlite::params![ContentType::SYSTEM, &literal_type],
    )
}

pub fn get_userpage(ctx: &PageContext, user: i64) -> Result<Option<BasicContent>, Error> {
    let query = format!(
        "SELECT {} FROM content WHERE id IN (SELECT MIN(id) FROM content WHERE {} AND contentType = ? AND createUserId = ?)",
        BASICCONTENTFIELDS,
        COMMONCONTENT
    );
    let mut stmt = ctx.dbcon.prepare(&query)?;
    #[cfg(feature = "querydump")]
    println!("Query: {:?}", &query);

    Ok(gather_basiccontent(&mut stmt, rusqlite::params![ContentType::USERPAGE, &user])?.pop())
}

pub fn get_basic_by_pid(ctx: &PageContext, pid: i64) -> Result<Option<BasicContent>, Error> {
    let query = format!(
        "SELECT {} FROM content WHERE {} AND id IN (SELECT contentId FROM content_values WHERE `key`=? AND value=?)",
        BASICCONTENTFIELDS, COMMONCONTENT
    );
    let mut stmt = ctx.dbcon.prepare(&query)?;
    #[cfg(feature = "querydump")]
    println!("Query: {:?}", &query);
    let spid = format!("{}", pid);
    Ok(gather_basiccontent(&mut stmt, rusqlite::params!["pid", &spid])?.pop())
}

pub fn get_msgid_by_cid(
    ctx: &PageContext,
    content_id: i64,
    cid: i64,
) -> Result<Option<i64>, Error> {
    let query = format!(
        "SELECT messageId FROM message_values WHERE `key`=? AND `value`=? AND messageId IN (SELECT id FROM messages WHERE contentId = ?)",
    );
    let scid = format!("{}", cid);
    let scontent_id = format!("{}", content_id);
    let mut stmt = ctx.dbcon.prepare(&query)?;
    let msgid_iter = stmt.query_map(rusqlite::params!["cid", &scid, &scontent_id], |row| {
        row.get::<usize, i64>(0)
    })?;

    #[cfg(feature = "querydump")]
    println!("Query: {:?}", &query);

    Ok(msgid_iter
        .collect::<Result<Vec<i64>, rusqlite::Error>>()?
        .pop())
}

#[derive(Clone, Debug)]
pub struct DocTreeContent {
    pub id: i64,
    pub hash: String,
    pub name: String,
    pub values: HashMap<String, String>,
}

// Get any system content with given literaltype
pub fn get_all_documentation(ctx: &PageContext) -> Result<Vec<DocTreeContent>, Error> {
    let query = format!(
        "SELECT id,hash,name FROM content WHERE {} AND contentType = ? AND literalType = ?",
        COMMONCONTENT
    );
    let mut stmt = ctx.dbcon.prepare(&query)?;
    let mut vstmt = ctx.dbcon.prepare(VALUESELECT)?;
    let doc_iter = stmt.query_map(
        rusqlite::params![ContentType::PAGE, SBSPageType::DOCUMENTATION],
        |row| {
            let id = row.get(0)?;
            Ok(DocTreeContent {
                id,
                hash: row.get(1)?,
                name: row.get(2)?,
                values: gather_values(&mut vstmt, id)?,
            })
        },
    )?;

    #[cfg(feature = "querydump")]
    println!("Query: {:?}", &query);

    Ok(doc_iter.collect::<Result<Vec<DocTreeContent>, rusqlite::Error>>()?)
}

#[derive(Clone, Debug)]
pub struct SearchAllUser {
    pub id: i64,
    pub username: String,
    pub avatar: String,
}

#[derive(Clone, Debug)]
pub struct SearchAllContent {
    pub id: i64,
    pub hash: String,
    pub name: String,
    pub literal_type: String,
    pub values: HashMap<String, String>,
    pub keywords: Vec<String>,
}

pub enum SearchAllResult {
    User(SearchAllUser),
    Content(SearchAllContent),
}

pub fn get_searchall(ctx: &PageContext, search: &str) -> Result<Vec<SearchAllResult>, Error> {
    let mut result: Vec<SearchAllResult> = Vec::new();
    let rsearch = if search.len() < 2 {
        format!("{}%", search) // Short search length means more optimized "begins with"
    } else {
        format!("%{}%", search)
    };

    // First, search users
    let query = format!(
        "SELECT id,username,avatar FROM users WHERE {} AND username LIKE ?",
        COMMONUSER
    );
    let mut stmt = ctx.dbcon.prepare(&query)?;
    let user_iter = stmt.query_map([&rsearch], |row| {
        Ok(SearchAllResult::User(SearchAllUser {
            id: row.get(0)?,
            username: row.get(1)?,
            avatar: row.get(2)?,
        }))
    })?;
    for u in user_iter {
        result.push(u?);
    }

    #[cfg(feature = "querydump")]
    println!("Query: {:?}", &query);

    // And then, content. It's a bit silly, but for I think slightly better performance,
    // we query the set of all content that matches the keywords separately from querying
    // the actual keyword list. There are better ways to do this, but I'm lazy, sorry
    // future self?
    let query = format!(
        "SELECT id,hash,name,COALESCE(literalType, '') AS lt FROM content WHERE {} AND literalType IN ({}) AND (name LIKE ? OR id IN (SELECT contentId FROM content_keywords WHERE `value` LIKE ?))",
        COMMONCONTENT,
        params_list(THREADTYPES.len())
    );

    let mut params: Vec<&dyn rusqlite::types::ToSql> = Vec::new();
    for tt in THREADTYPES.iter() {
        params.push(tt);
    }
    params.push(&rsearch);
    params.push(&rsearch);

    let mut stmt = ctx.dbcon.prepare(&query)?;
    let mut vstmt = ctx.dbcon.prepare(VALUESELECT)?;
    let mut kstmt = ctx.dbcon.prepare(KEYWORDSELECT)?;
    let content_iter = stmt.query_map(params.as_slice(), |row| {
        let id: i64 = row.get(0)?;
        Ok(SearchAllResult::Content(SearchAllContent {
            id,
            hash: row.get(1)?,
            name: row.get(2)?,
            literal_type: row.get(3)?,
            values: gather_values(&mut vstmt, id)?,
            keywords: gather_keywords(&mut kstmt, id)?,
        }))
    })?;
    for c in content_iter {
        result.push(c?);
    }

    #[cfg(feature = "querydump")]
    println!("Query: {:?}", &query);

    Ok(result)
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
    let mut query = format!("SELECT id,name,hash,(SELECT `value` FROM content_values WHERE contentId=c.id AND `key`=?) FROM content AS c WHERE {} AND contentType = ? AND literalType = ?",
        COMMONCONTENT,
    );

    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![
        Box::new(SBSValue::FORCONTENT),
        Box::new(ContentType::SYSTEM),
        Box::new(SBSPageType::CATEGORY),
    ];

    if let Some(ids) = ids {
        query.push_str(&format!(" AND id IN ({})", params_list(ids.len())));
        for id in ids {
            params.push(Box::new(id));
        }
    }

    let mut stmt = ctx.dbcon.prepare(&query)?;
    let cat_iter = stmt.query_map(box_to_ref!(params), |row| {
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
    })?;

    #[cfg(feature = "querydump")]
    println!("Query: {:?}", &query);

    Ok(cat_iter.collect::<Result<Vec<SubmissionCategory>, rusqlite::Error>>()?)
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
    stmt: &mut rusqlite::Statement,
    vstmt: &mut Option<rusqlite::Statement>,
    params: &[&dyn rusqlite::ToSql],
) -> Result<Vec<BrowseContent>, Error> {
    let browse_iter = stmt.query_map(params, |row| {
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
    })?;

    return Ok(browse_iter.collect::<Result<Vec<BrowseContent>, rusqlite::Error>>()?);
}

pub fn get_browse(
    ctx: &PageContext,
    search: &forms::PageSearch,
    limits: QueryLimit,
) -> Result<Vec<BrowseContent>, Error> {
    let mut query = format!(
        r##"SELECT {}, (SELECT COUNT(*) FROM content_engagement WHERE contentId=c.id AND `type`=? AND engagement = ?) AS upvotes 
        FROM content AS c WHERE {} AND contentType=? AND parentId IN
            (SELECT id FROM content WHERE contentType = ? AND literalType = ?)"##,
        BROWSEFIELDS, COMMONCONTENT,
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
        query.push_str(" AND (name LIKE ? OR c.id IN (SELECT contentId FROM content_keywords WHERE `value` LIKE ?))");
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
            query.push_str(" AND createUserId = ?");
        }
    }

    if let Some(subtype) = &search.subtype {
        if !subtype.is_empty() {
            params.push(Box::new(subtype.clone()));
            query.push_str(" AND literalType = ?");
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

    if search.order == "id" {
        query.push_str(" ORDER BY id");
    } else if search.order == "id_desc" {
        query.push_str(" ORDER BY id DESC");
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

    #[cfg(feature = "querydump")]
    println!("Query: {:?}", &query);

    gather_browsecontent(&mut stmt, &mut Some(vstmt), box_to_ref!(params))
}

pub fn get_badges(ctx: &PageContext, uid: i64) -> Result<Vec<BrowseContent>, Error> {
    let query = format!(
        "SELECT {} FROM content AS c WHERE {} AND contentType=? AND c.id IN (SELECT relatedId FROM user_relations WHERE userId = ? AND type = ?)",
        BROWSEFIELDS,
        COMMONCONTENT,
    );

    let mut stmt = ctx.dbcon.prepare(&query)?;

    #[cfg(feature = "querydump")]
    println!("Query: {:?}", &query);

    gather_browsecontent(
        &mut stmt,
        &mut None,
        rusqlite::params![&ContentType::FILE, &uid, &UserRelationType::ASSIGNCONTENT],
    )
}

#[derive(Clone, Debug)]
pub struct QrPageData {
    pub id: i64,
    pub hash: String,
    pub name: String,
    pub qr_raw: Option<String>,
}

pub fn get_qrpage(ctx: &PageContext, hash: &str) -> Result<QrPageData, Error> {
    let query = format!("SELECT id,hash,name,(SELECT `text` FROM content cc WHERE {} AND cc.parentId=c.id AND cc.literalType = ?) FROM content AS c WHERE {} AND c.hash = ?",
        COMMONCONTENT,
        COMMONCONTENT,
    );
    let mut stmt = ctx.dbcon.prepare(&query)?;
    let qr_iter = stmt.query_map(rusqlite::params![&PTCSYSTEM, &hash], |row| {
        //let id: i64 = row.get(0)?;
        Ok(QrPageData {
            id: row.get(0)?,
            hash: row.get(1)?,
            name: row.get(2)?,
            qr_raw: row.get(3)?,
        })
    })?;

    #[cfg(feature = "querydump")]
    println!("Query: {:?}", &query);

    qr_iter
        .collect::<Result<Vec<QrPageData>, rusqlite::Error>>()?
        .pop()
        .ok_or(Error::NotFound(String::from(hash)))
}

//pub fn get_

// pub async fn get_pid_redirect(context: PageContext) -> Result<String, Error> {
//     let mut request = FullRequest::new();
//     add_value!(request, "pidkey", vec!["pid"]);
//     add_value!(request, "pid", vec![query.pid]);
//
//     //Basically: go look for the content that has the given pid
//     let pid_request = build_request!(
//         RequestType::content,
//         String::from("id,hash,values"),
//         format!("!valuein(@pidkey, @pid)")
//     );
//     request.requests.push(pid_request);
//
//     if let Some(cid) = query.cid {
//         add_value!(request, "cidkey", vec!["cid"]);
//         add_value!(request, "cid", vec![cid]);
//         let cid_request = build_request!(
//             RequestType::message,
//             String::from("id,values,contentId"),
//             format!("!valuein(@cidkey, @cid)")
//         );
//         request.requests.push(cid_request);
//     }
//
//     let result = context.api_context.post_request(&request).await?;
//     let mut pages = cast_result_required::<Content>(&result, "content")?;
//     let mut messages = cast_result_safe::<Message>(&result, "message")?;
//
//     let page = pages
//         .pop()
//         .ok_or(Error::NotFound(String::from("Could not find page!")))?;
//
//     let mut url = context.layout_data.links.forum_thread(&page);
//
//     if let Some(message) = messages.pop() {
//         url = context.layout_data.links.forum_post(&message, &page);
//     }
//
//     Ok(Response::Redirect(url))
// }
