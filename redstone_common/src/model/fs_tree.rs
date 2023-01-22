use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, path::PathBuf};

use crate::util::generate_sha256_digest;

use super::{ArgumentError, RedstoneError, Result};

type Sha256Digest = String;

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RSFile {
    pub path: String,
    pub sha_256_digest: String,
    pub depth: u16,
    pub size: u64,
}

impl RSFile {
    pub fn new(path: String, sha_256_digest: String, depth: u16, size: u64) -> Self {
        Self {
            path,
            sha_256_digest,
            depth,
            size,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RSFolder {
    pub path: String,
    pub items: Vec<RSFile>,
    pub depth: u16,
}

impl RSFolder {
    pub fn new(path: String, items: Vec<RSFile>, depth: u16) -> Self {
        Self { path, items, depth }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct FSTree {
    pub files: Vec<RSFile>,
    pub root: PathBuf,
    pub max_depth: Option<u16>,
}

impl FSTree {
    pub fn build(root: PathBuf, max_depth: Option<u16>) -> Result<Self> {
        let root_is_file = root.is_file();
        let mut fs_tree = FSTree {
            root,
            files: Vec::new(),
            max_depth,
        };

        let root_as_string = &fs_tree.root.to_str().unwrap();
        if root_is_file {
            return Err(RedstoneError::ArgumentError(
                ArgumentError::PathCannotBeAFile(String::from(*root_as_string)),
            ));
        }
        fs_tree.files = read_dir(&fs_tree.root, 0, max_depth, root_as_string, &mut Vec::new())?;

        Ok(fs_tree)
    }

    pub fn get_first_depth(&self) -> Vec<&RSFile> {
        let mut items: Vec<&RSFile> = self.files.iter().filter(|item| item.depth <= 1).collect();
        items.sort_by(|a, b| b.depth.partial_cmp(&a.depth).unwrap());
        items
    }

    pub fn total_size(&self) -> u64 {
        self.files.iter().map(|file| file.size).sum()
    }
}

fn read_dir(
    dir: &PathBuf,
    depth: u16,
    max_depth: Option<u16>,
    root: &str,
    ignores: &mut Vec<String>,
) -> Result<Vec<RSFile>> {
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
        if path.is_dir() && path != *dir {
            file_tree_items.extend(read_dir(&path, depth + 1, max_depth, root, ignores)?);
        } else if path.is_file() {
            let file_path = build_relative_file_path(&path, root);
            let sha256_digest: Sha256Digest = generate_sha256_digest(&path).unwrap();
            let size = std::fs::metadata(&path)?.len();
            let file = RSFile::new(file_path, sha256_digest, depth, size);
            file_tree_items.push(file);
        }
    }
    return Ok(file_tree_items);
}
fn build_relative_file_path(path: &PathBuf, root: &str) -> String {
    let suffix = match root.ends_with("/") {
        true => String::from(root),
        false => String::from(root) + "/",
    };
    String::from(remove_prefix(path.to_str().unwrap(), &suffix))
}

fn remove_prefix<'a>(s: &'a str, suffix: &str) -> &'a str {
    match s.strip_prefix(suffix) {
        Some(s) => s,
        None => s,
    }
}

#[cfg(test)]
mod tests {
    use crate::model::fs_tree::RSFile;
    use std::{path::PathBuf, str::FromStr};

    use super::FSTree;

    #[test]
    fn scans_a_directory_recursively() {
        let path = PathBuf::from_str("./test-data").unwrap();
        let mut fs_tree = FSTree::build(path.clone(), None).unwrap();
        let files = vec![
            RSFile::new(
                String::from("./test-data/other_folder/other_file.hs"),
                String::from("982bc87271bad527f4659eb12ecf1fd1295ae9fe0acfcfc83539fb9c0e523f64"),
                0,
                200 as u64,
            ),
            RSFile::new(
                String::from("./test-data/other_folder/other_file.hs"),
                String::from("982bc87271bad527f4659eb12ecf1fd1295ae9fe0acfcfc83539fb9c0e523f64"),
                0,
                200 as u64,
            ),
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
