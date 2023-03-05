
use std::{collections::HashMap, cell::RefCell};

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
    pub tree_nodes : HashMap<String, DocTreeNode<'a>>, 
    pub page_nodes : Vec<&'a Content> 
}

impl<'a> DocTreeNode<'a> 
{
    pub fn new(name: &str) -> Self {
        DocTreeNode { 
            name: name.to_string(), 
            tree_nodes: HashMap::new(), 
            page_nodes: Vec::new() 
        }
    }

    pub fn fill_path(&mut self, path: &[&str]) -> &'a mut DocTreeNode {
        if let Some(part) = path.get(0) {
            //OK this is the next part of the path. We need to find something inside ourselves or add it if not
            if self.tree_nodes.contains_key(*part) { //let Some(node) = self.tree_nodes.get_mut(*part) {
                self.tree_nodes.get_mut(*part).unwrap().fill_path(&path[1..])
            }
            else {
                self.tree_nodes.insert(part.to_string(), DocTreeNode::new(part));
                self.tree_nodes.get_mut(*part).unwrap().fill_path(&path[1..])
            }
        }
        else {
            //There's no more path, we are it
            self
        }
    }

    //pub fn get_or_add_named_node(&mut self, name: &str) -> &'a DocTreeNode {
    //    if let Some(existing) = self.tree_nodes.borrow().iter().find(|x| x.name == name) {
    //        existing
    //    }
    //    else {
    //        let new_node = DocTreeNode::new(name);
    //        self.tree_nodes.borrow_mut().push(new_node);
    //        self.tree_nodes.borrow().last().unwrap()
    //    }
    //}

    //Path's first element should not be the node itself, but the next node in the path. For instance, if you
    //are at a "root" node, the path should start with "SmileBASIC 4" (if we're talking documentation)
    ///// Add the given page nodes to this doc tree node, or fill out the missing tree links and add them deeper.
    //pub fn add_pagenodes(&mut self, path: &Vec<&str>, nodes: Vec<&'a Content>) { //} -> &'a DocTreeNode {
    
    //pub fn add_pagenodes(&mut self, path: Vec<&str>, content: Vec<&'a Content>) { //} -> &'a DocTreeNode {
    //    for part in path {

    //    }
    //    //If there's still a path, we want to find the next node to operate on. 
    //    if let Some(part) = path.get(0) {
    //        let new_path = path.iter().skip(1).map(|x| *x).collect::<Vec<&str>>();
    //        let node = self.get_or_add_named_node(part);
    //        node.add_pagenodes(&new_path, nodes);
    //    }
    //    else { //The path is empty, so this must be the current node!
    //        self.page_nodes.borrow_mut().extend(nodes);
    //        //self
    //    }
    //}
}

/// Build a document tree and return the root node, which you can use to traverse the whole tree. The root node
/// has no name, and all other actual roots go below (since there could be multiple, such as SB4, SB3, etc)
pub fn get_doctree<'a>(documentation: &'a Vec<Content>) -> DocTreeNode<'a>
{
    //Easiest to just pre-compute the paths (it's a little wasteful but whatever)
    let docpaths = get_all_docpaths(documentation);

    let root_node = RefCell::new(DocTreeNode::default()); //RefCell::new(DocTreeNode::default());

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

            root_node.borrow_mut().fill_path(&path_parts[1..]);
            //for part in path_parts.into_iter().skip(1) {

            //}

            //let mut mut_root = root_node.borrow_mut();
            //mut_root.add_pagenodes(&path_parts.into_iter().skip(1).collect(), content);
            //AND THEN IT GOES OUT OF SCOPE, COME ONNNN
        }
        else {
            println!("{} DOCUMENTATION DROPPED WITH EMPTY PATH", content.len());
            continue;
        }
    }

    //let a = root_node.take();
    //a
    root_node.take()
}