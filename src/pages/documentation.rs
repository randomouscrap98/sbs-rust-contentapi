use maud::*;
use std::collections::HashMap;

use crate::layout::*;
use crate::pages::common::contentapi::*;
use crate::pages::context::*;
use crate::response::*;

// ---------------------------
//       CONTENTAPI
// ---------------------------

#[derive(Clone, Debug)]
struct DocTreeContent {
    //pub id: i64,
    pub hash: String,
    pub name: String,
    pub values: HashMap<String, String>,
}

// Get any system content with given literaltype
fn get_all_documentation(ctx: &PageContext) -> Result<Vec<DocTreeContent>, Error> {
    let query = format!(
        "SELECT hash,name FROM content WHERE {} AND contentType = ? AND literalType = ?",
        COMMONCONTENT
    );
    let mut stmt = ctx.dbcon.prepare(&query)?;
    let mut vstmt = ctx.dbcon.prepare(VALUESELECT)?;
    easy_query(
        (&mut stmt, &query),
        rusqlite::params![ContentType::PAGE, SBSPageType::DOCUMENTATION],
        |row| {
            let id = row.get(0)?;
            Ok(DocTreeContent {
                //id,
                hash: row.get(1)?,
                name: row.get(2)?,
                values: gather_values(&mut vstmt, id)?,
            })
        },
    )
}

// ---------------------------
//       RENDER
// ---------------------------

#[derive(Default, Clone)]
struct DocTreeNode<'a> {
    pub name: String,
    pub tree_nodes: Vec<DocTreeNode<'a>>,
    pub page_nodes: Vec<&'a DocTreeContent>,
}

impl<'a> DocTreeNode<'a> {
    pub fn new(name: &str) -> Self {
        DocTreeNode {
            name: name.to_string(),
            tree_nodes: Vec::new(),
            page_nodes: Vec::new(),
        }
    }

    pub fn add_content_fill_path(&mut self, path: &[&str], nodes: Vec<&'a DocTreeContent>) {
        if let Some(part) = path.get(0) {
            //OK this is the next part of the path. We need to find something inside ourselves or add it if not
            if self.tree_nodes.iter().any(|n| n.name == *part) {
                //let Some(node) = self.tree_nodes.get_mut(*part) {
                self.tree_nodes
                    .iter_mut()
                    .find(|n| n.name == *part)
                    .unwrap()
                    .add_content_fill_path(&path[1..], nodes);
            } else {
                self.tree_nodes.push(DocTreeNode::new(part));
                self.tree_nodes
                    .last_mut()
                    .unwrap()
                    .add_content_fill_path(&path[1..], nodes);
            }
        } else {
            //There's no more path, we are it
            self.page_nodes.extend(nodes);
        }
    }
}

fn walk_doctree_recursive(
    layout_data: &MainLayoutData,
    tree: &DocTreeNode,
    open_levels: i32,
) -> Markup {
    let mut tree_nodes = tree.tree_nodes.clone();
    tree_nodes.sort_by(|a, b| a.name.cmp(&b.name));

    html! {
        details."docnode" open[open_levels > 0] {
            summary { (tree.name) }
            @for node in &tree_nodes {
                (walk_doctree_recursive(layout_data, node, open_levels - 1))
            }
            ul {
                @for content in &tree.page_nodes {
                    li { a."flatlink" href=(layout_data.links.forum_thread_unsafe(&content.hash)) { (content.name) } }
                }
            }
        }
    }
}

fn walk_doctree(layout_data: &MainLayoutData, tree: &DocTreeNode, open_levels: i32) -> Markup {
    let mut tree_nodes = tree.tree_nodes.clone();
    tree_nodes.sort_by(|a, b| a.name.cmp(&b.name));

    //For the root node, we only list the treenodes and NOT ourselves. Then normal recursion
    html! {
        @for node in &tree_nodes {
            (walk_doctree_recursive(layout_data, node, open_levels))
        }
    }
}

/// Map content into paths containing the list of content within each path. Can be used to later
/// build a tree, or to get a list of all paths (they keys of result)
fn get_all_docpaths(documentation: &Vec<DocTreeContent>) -> HashMap<String, Vec<&DocTreeContent>> {
    let mut result: HashMap<String, Vec<&DocTreeContent>> = HashMap::new();

    for doc in documentation {
        // println!("Values: {:?}", doc.values);
        if let Some(docpath) = doc.values.get(SBSValue::DOCPATH) {
            //println!("Docpath: {}", docpath);
            match serde_json::from_str::<&str>(docpath) {
                Ok(v) => {
                    if let Some(list) = result.get_mut(v) {
                        list.push(doc);
                    } else {
                        result.insert(v.to_string(), vec![doc]);
                    }
                }
                Err(e) => {
                    println!("Can't parse docpath: {}", e);
                }
            }
        } else {
            println!("WARNING: documentation {} didn't have a docpath!", doc.name);
        }
    }

    result
}

/// Build a document tree and return the root node, which you can use to traverse the whole tree. The root node
/// has no name, and all other actual roots go below (since there could be multiple, such as SB4, SB3, etc)
fn get_doctree<'a>(documentation: &'a Vec<DocTreeContent>) -> DocTreeNode<'a> {
    //Easiest to just pre-compute the paths (it's a little wasteful but whatever)
    let docpaths = get_all_docpaths(documentation);

    // println!("Docpaths used for tree: {:#?}", docpaths);

    let mut root_node = DocTreeNode::default();

    for (path, content) in docpaths {
        //println!("Parsing {} with {} content", path, content.len());
        //Split the path up into parts
        let path_parts = path.split("/").collect::<Vec<&str>>();

        if let Some(root_path) = path_parts.get(0) {
            //This indicates the path did NOT start with /, meaning we don't know where to place it. We COULD make an assumption I guess...
            //but I'll wait until later to do that
            if !root_path.is_empty() {
                println!(
                    "{} DOCUMENTATION DROPPED WITH NON-ROOTED PATH",
                    content.len()
                );
                continue;
            }

            root_node.add_content_fill_path(&path_parts[1..], content);
        } else {
            println!("{} DOCUMENTATION DROPPED WITH EMPTY PATH", content.len());
            continue;
        }
    }

    root_node
}

fn display_doctree(
    layout_data: &MainLayoutData,
    documentation: Vec<DocTreeContent>,
    open_levels: i32,
) -> Markup {
    html! {
        div."documenttree" {
            (walk_doctree(layout_data, &mut get_doctree(&documentation), open_levels))
        }
    }
}

//This will render the entire index! It's a handler WITH the template in it! Maybe that's kinda weird? who knows...
//pub fn index(data: MainLayoutData) -> Result<impl warp::Reply, Infallible>{
fn render(data: MainLayoutData, documentation: Vec<DocTreeContent>) -> String {
    layout(
        &data,
        html! {
            (data.links.style("/forpage/forum.css"))
            (data.links.script("/forpage/forum.js"))
            section {
                (display_doctree(&data, documentation, 1))
            }
        },
    )
    .into_string()
}

pub async fn get_render(context: PageContext) -> Result<Response, Error> {
    let documentation = get_all_documentation(&context)?; //get_all_documentation(&mut context.api_context).await?;
    Ok(Response::Render(render(context.layout_data, documentation)))
}
