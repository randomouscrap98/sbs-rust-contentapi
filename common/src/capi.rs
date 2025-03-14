use chrono::{DateTime, Utc};

use super::*;
use crate::constants::*;
use crate::response::*;

pub static CATEGORYFIELDS2: &str = "id,hash,name,COALESCE(description,'') AS description,COALESCE(literalType,'') AS lt,contentType";
pub static THREADFIELDS2: &str =
    "id,hash,name,COALESCE(literalType,''),contentType,parentId,createDate,createUserId";
pub static COMMONCONTENT: &str =
    " deleted = 0 AND id IN (SELECT contentId FROM content_permissions WHERE read=1 AND userId=0) ";
pub static COMMONUSER: &str =
    " deleted = 0 AND `type`= 1 AND (registrationKey IS NULL OR registrationKey = '') ";
pub static VALUESELECT: &str = "SELECT `key`,`value` FROM content_values WHERE contentId=?";
pub static KEYWORDSELECT: &str = "SELECT `value` FROM content_keywords WHERE contentId=?";
//pub static SUBMISSIONPARENT: &str = "SELECT id FROM content WHERE contentId=?";

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
    let query = format!("SELECT id,`type`,username,avatar,special,super,createDate FROM users WHERE {} AND id IN ({})",
        COMMONUSER,
        params_list(ids.len()),
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

pub fn get_forum_categories(
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
pub struct SystemPage {
    pub id: i64,
    pub hash: String,
    pub name: String,
    pub text: String,
}

// Get any system content with given literaltype
pub fn get_systempage(ctx: &PageContext, literal_type: String) -> Result<Vec<SystemPage>, Error> {
    let query = format!(
        "SELECT id,hash,name,text FROM content WHERE {} AND contentType = ? AND literalType = ?",
        COMMONCONTENT
    );
    let mut stmt = ctx.dbcon.prepare(&query)?;
    let system_iter = stmt.query_map(
        rusqlite::params![ContentType::SYSTEM, &literal_type],
        |row| {
            Ok(SystemPage {
                id: row.get(0)?,
                hash: row.get(1)?,
                name: row.get(2)?,
                text: row.get(3)?,
            })
        },
    )?;

    #[cfg(feature = "querydump")]
    println!("Query: {:?}", &query);

    Ok(system_iter.collect::<Result<Vec<SystemPage>, rusqlite::Error>>()?)
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

#[derive(Debug)]
pub struct SubmissionCategory {
    pub id: i64,
    pub name: String,
    pub forcontent: String,
}

pub fn get_submission_categories(ctx: &PageContext) -> Result<Vec<SubmissionCategory>, Error> {
    let query = format!("SELECT id,name,(SELECT `value` FROM content_values WHERE contentId=c.id AND `key`=?) FROM content AS c WHERE {} AND contentType = ? AND literalType = ?",
        COMMONCONTENT,
    );

    let mut stmt = ctx.dbcon.prepare(&query)?;
    let cat_iter = stmt.query_map(
        rusqlite::params![
            SBSValue::FORCONTENT,
            &ContentType::SYSTEM,
            &SBSPageType::CATEGORY
        ],
        |row| {
            let cval: Option<String> = row.get(2)?;
            Ok(SubmissionCategory {
                id: row.get(0)?,
                name: row.get(1)?,
                forcontent: if let Some(cval) = cval {
                    serde_json::from_str(&cval).unwrap_or_default()
                } else {
                    String::new()
                },
            })
        },
    )?;

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

pub fn get_browse(
    ctx: &PageContext,
    search: &forms::PageSearch,
    limits: QueryLimit,
) -> Result<Vec<BrowseContent>, Error> {
    let mut query = format!(
        r##"SELECT id,hash,name,COALESCE(description,''),COALESCE(literalType,''),createDate,createUserId,
            (SELECT COUNT(*) FROM content_engagement WHERE contentId=c.id AND `type`=? AND engagement = ?) AS upvotes 
        FROM content AS c WHERE {} AND contentType=? AND parentId IN
            (SELECT id FROM content WHERE contentType = ? AND literalType = ?)"##,
        COMMONCONTENT,
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
                    //add_value!(request, "dlkeylist", vec![SBSValue::DOWNLOADKEY]);
                    query.push_str(
                        " AND c.id IN (SELECT contentId FROM content_values WHERE `key`= ? OR (`key` = ? AND `value` LIKE ?))" //(!valuekeyin(@dlkeylist) or !valuelike(@systemkey, @ptcsystem))",
                    );
                }
                if search.system != ANYSYSTEM {
                    params.push(Box::new(SBSValue::SYSTEMS));
                    params.push(Box::new(format!("%{}%", search.system)));
                    query.push_str(" AND c.id IN (SELECT contentId FROM content_values WHERE `key`=? AND `value` LIKE ?)");
                    //) !valuelike(@systemkey, @system)");
                    //add_value!(request, "system", format!("%{}%", search.system)); //Systems is actually a json list but this should be fine
                }
            }

            //add_value!(request, "systemkey", SBSValue::SYSTEMS);
            //add_value!(request, "ptcsystem", format!("%{}%", PTCSYSTEM));
            //Ignore certain search criteria
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
    let mut vstmt = ctx.dbcon.prepare(VALUESELECT)?;
    let browse_iter = stmt.query_map(
        params
            .iter()
            .map(|x| x.as_ref())
            .collect::<Vec<_>>()
            .as_slice(),
        |row| {
            let id: i64 = row.get(0)?;
            Ok(BrowseContent {
                id,
                hash: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                literal_type: row.get(4)?,
                create_date: row.get(5)?,
                create_user_id: row.get(6)?,
                values: gather_values(&mut vstmt, id)?,
            })
        },
    )?;

    #[cfg(feature = "querydump")]
    println!("Query: {:?}", &query);

    return Ok(browse_iter.collect::<Result<Vec<BrowseContent>, rusqlite::Error>>()?);
}
