use ignore::{Error, WalkBuilder};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, path::PathBuf};

use crate::util::generate_sha256_digest;

type Sha256Digest = String;

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum FSTreeItem {
    Folder(RSFolder),
    File(RSFile),
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RSFile {
    pub path: String,
    pub sha_256_digest: String,
    pub depth: u16,
}

impl RSFile {
    pub fn new(path: String, sha_256_digest: String, depth: u16) -> Self {
        Self {
            path,
            sha_256_digest,
            depth,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RSFolder {
    pub path: String,
    pub items: Vec<FSTreeItem>,
    pub depth: u16,
}

impl RSFolder {
    pub fn new(path: String, items: Vec<FSTreeItem>, depth: u16) -> Self {
        Self { path, items, depth }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct FSTree {
    pub files: Vec<FSTreeItem>,
    pub root: PathBuf,
    pub max_depth: Option<u16>,
}

impl FSTree {
    pub fn new(root: PathBuf, max_depth: Option<u16>) -> Self {
        let mut fs_tree = FSTree {
            root,
            files: Vec::new(),
            max_depth,
        };
        let root_as_string = &fs_tree.root.to_str().unwrap();
        fs_tree.files = read_dir(&fs_tree.root, 0, max_depth, root_as_string, &mut Vec::new()).unwrap();

        fs_tree
    }
    pub fn get_index_file_for_root(&self) -> PathBuf {
        let mut path = self.root.clone();
        to_index_path(&mut path);
        path
    }

    pub fn get_first_depth(&self) -> Vec<&FSTreeItem> {
        let mut items: Vec<&FSTreeItem> = self
            .files
            .iter()
            .filter(|item| match item {
                FSTreeItem::File(file) => file.depth <= 1,
                FSTreeItem::Folder(folder) => folder.depth <= 1,
            })
            .collect();
        items.sort();
        items
    }
}

pub fn to_index_path(path: &mut PathBuf) {
    path.push(".rs");
    path.push("index");
}

fn read_dir(
    dir: &PathBuf,
    depth: u16,
    max_depth: Option<u16>,
    root: &str,
    ignores: &mut Vec<String>,
) -> Result<Vec<FSTreeItem>, Error> {
    let mut file_tree_items = Vec::new();
    if max_depth.is_some() && depth > max_depth.unwrap() {
        return Ok(file_tree_items);
    }

    {
        let mut dir = dir.clone();
        dir.push(".rsignore");
        if dir.exists() {
            ignores.push(String::from(dir.to_str().unwrap()));
        }
        dir.push("..");
    }

    let mut builder_base = WalkBuilder::new(&dir);
    for previous_ignore in ignores.iter_mut() {
        builder_base.add_custom_ignore_filename(previous_ignore);
    }

    for entry in builder_base
        .standard_filters(false)
        .hidden(false)
        .max_depth(Some(1))
        .build()
    {
        let entry = entry?;
        let path = PathBuf::from(entry.path());
        let file_path = String::from(
            remove_prefix(
                path.to_str().unwrap(),
                root
            )
        );

        if path.is_dir() && path != *dir {
            let entry_content = read_dir(&path, depth + 1, max_depth, root, ignores)?;
            let folder = RSFolder::new(file_path, entry_content, depth);
            file_tree_items.push(FSTreeItem::Folder(folder));
        } else if path.is_file() {
            let sha256_digest: Sha256Digest = generate_sha256_digest(&path).unwrap();
            let file = RSFile::new(file_path, sha256_digest, depth);
            file_tree_items.push(FSTreeItem::File(file));
        }
    }
    return Ok(file_tree_items);
}

#[cfg(test)]
mod tests {
    use crate::model::fs_tree::{RSFile, RSFolder};
    use std::{path::PathBuf, str::FromStr};

    use super::{FSTree, FSTreeItem};

    #[test]
    fn scans_a_directory_recursively() {
        let path = PathBuf::from_str("./test-data").unwrap();
        let mut fs_tree = FSTree::new(path.clone(), None);
        let files = vec![
            FSTreeItem::Folder(RSFolder::new(
                String::from("./test-data/other_folder"),
                vec![FSTreeItem::File(RSFile::new(
                    String::from("./test-data/other_folder/other_file.hs"),
                    String::from(
                        "982bc87271bad527f4659eb12ecf1fd1295ae9fe0acfcfc83539fb9c0e523f64",
                    ),
                    0,
                ))],
                0,
            )),
            FSTreeItem::File(RSFile::new(
                String::from("./test-data/hello.ex"),
                String::from("1d8326dc32bc35812503ecfcce8ca3db0f025fb84d589df4e687b96f6cdf03fe"),
                0,
            )),
        ];
        let mut target_fs_tree = FSTree {
            files,
            root: path.clone(),
            max_depth: None,
        };
        assert_eq!(target_fs_tree.max_depth, fs_tree.max_depth);
        assert_eq!(target_fs_tree.root, fs_tree.root);
        assert_eq!(target_fs_tree.files.sort(), fs_tree.files.sort());
    }
}

fn remove_prefix<'a>(s: &'a str, suffix: &str) -> &'a str {
    match s.strip_prefix(suffix) {
        Some(s) => s,
        None => s
    }
}
