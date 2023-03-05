
use std::{collections::HashMap};

use contentapi::*;
use crate::{constants::*, opt_s};

// This is for NON-API basic data conversion / organization related to views.

/// Get the list of category ids this content is tagged under
pub fn get_tagged_categories(content: &Content) -> Vec<i64>
{
    let mut result : Vec<i64> = Vec::new();

    if let Some(ref values) = content.values {
        for (key, _value) in values {
            if key.starts_with(CATEGORYPREFIX) {
                if let Ok(category) = (&key[CATEGORYPREFIX.len()..]).parse::<i64>() {
                    result.push(category)
                }
            }
        }
    }

    result
}

/// Add a parsed list of categories from a user form (which should be just ids)
/// to the given content. It will add them as values
pub fn add_category_taglist(raw_parsed: Vec<String>, content: &mut Content)
{
    if let Some(ref mut values) = content.values {
        for category in raw_parsed {
            values.insert(format!("{}{}", CATEGORYPREFIX, category), true.into());
        }
    }
}

pub fn get_thumbnail_hash(content: &Content) -> Option<String>
{
    if let Some(ref values) = content.values {
        if let Some(ref images) = values.get(SBSValue::IMAGES).and_then(|k| k.as_array()) {
            if let Some(image) = images.get(0).and_then(|i| i.as_str()) {
                return Some(image.to_string())
            }
        }
    }

    None
}


/// Get the list of all SBS systems listed in this content
pub fn get_systems(content: &Content) -> Vec<String>
{
    let mut result = Vec::new();

    if let Some(ref values) = content.values {
        if let Some(systems) = values.get(SBSValue::SYSTEMS).and_then(|k| k.as_array()) {
            for system in systems {
                if let Some(sys) = system.as_str() {
                    result.push(sys.to_string());
                }
            }
        }
    }

    return result;
}

#[derive(Debug)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub forcontent: String
}

pub fn map_categories(categories: Vec<Content>) -> Vec<Category>
{
    categories.into_iter().map(|c| {
        Category {
            id: c.id.unwrap_or(0),
            name: c.name.unwrap_or_else(|| String::from("")), //Only evaluated on failure
            forcontent: c.values
                .and_then(|v| v.get(SBSValue::FORCONTENT).and_then(|v2| v2.as_str()).and_then(|v3| Some(String::from(v3))))
                .unwrap_or_else(|| String::from(""))
        }
    }).collect::<Vec<Category>>()
}


// ----------------------
//    DOCUMENTATION
// ----------------------

/// Map content into paths containing the list of content within each path. Can be used to later
/// build a tree, or to get a list of all paths (they keys of result)
pub fn get_all_docpaths(documentation: &Vec<Content>) -> HashMap<String, Vec<&Content>>
{
    let mut result : HashMap<String, Vec<&Content>> = HashMap::new();

    for doc in documentation {
        //Go through the absurd unwrapping to get to the actual docpath
        if let Some(ref values) = doc.values {
            if let Some(docpath) = values.get(SBSValue::DOCPATH) {
                if let Some(docpath) = docpath.as_str() {
                    //Finally, either add the content to the list or insert a new key if the hashmap didn't have it
                    if let Some(list) = result.get_mut(docpath) {
                        list.push(doc);
                    }
                    else {
                        result.insert(docpath.to_string(), vec![doc]);
                    }
                    continue;
                }
            }
        }
        println!("WARNING: documentation {} didn't have a docpath!", opt_s!(doc.name));
    }

    result
}

/// A single node in a document tree. Each node contains 0 or more subsequent tree nodes to go deeper,
/// and 0 or more immediate page leaves. If the tree is build directly from documentation, the tree should
/// not have any nodes with no content.
#[derive(Default)]
pub struct DocTreeNode<'a> {
    pub name : String,
    pub tree_nodes : Vec<DocTreeNode<'a>>, 
    pub page_nodes : Vec<&'a Content> 
}

impl<'a> DocTreeNode<'a> 
{
    pub fn new(name: &str) -> Self {
        DocTreeNode { 
            name: name.to_string(), 
            tree_nodes: Vec::new(), 
            page_nodes: Vec::new() 
        }
    }

    pub fn add_content_fill_path(&mut self, path: &[&str], nodes: Vec<&'a Content>) {
        if let Some(part) = path.get(0) {
            //OK this is the next part of the path. We need to find something inside ourselves or add it if not
            if self.tree_nodes.iter().any(|n| n.name == *part) { //let Some(node) = self.tree_nodes.get_mut(*part) {
                self.tree_nodes.iter_mut().find(|n| n.name == *part).unwrap().add_content_fill_path(&path[1..], nodes);
            }
            else {
                self.tree_nodes.push(DocTreeNode::new(part));
                self.tree_nodes.last_mut().unwrap().add_content_fill_path(&path[1..], nodes);
            }
        }
        else {
            //There's no more path, we are it
            self.page_nodes.extend(nodes);
        }
    }
}

/// Build a document tree and return the root node, which you can use to traverse the whole tree. The root node
/// has no name, and all other actual roots go below (since there could be multiple, such as SB4, SB3, etc)
pub fn get_doctree<'a>(documentation: &'a Vec<Content>) -> DocTreeNode<'a>
{
    //Easiest to just pre-compute the paths (it's a little wasteful but whatever)
    let docpaths = get_all_docpaths(documentation);

    println!("Docpaths used for tree: {:#?}", docpaths);

    let mut root_node = DocTreeNode::default();

    for (path, content) in docpaths {
        //Split the path up into parts
        let path_parts = path.split("/").collect::<Vec<&str>>();

        if let Some(root_path) = path_parts.get(0) {
            //This indicates the path did NOT start with /, meaning we don't know where to place it. We COULD make an assumption I guess...
            //but I'll wait until later to do that
            if !root_path.is_empty() { 
                println!("{} DOCUMENTATION DROPPED WITH NON-ROOTED PATH", content.len());
                continue;
            }

            root_node.add_content_fill_path(&path_parts[1..], content);
        }
        else {
            println!("{} DOCUMENTATION DROPPED WITH EMPTY PATH", content.len());
            continue;
        }
    }

    root_node
}