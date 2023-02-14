use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr,
    fmt::Debug,
    path::{Path, PathBuf},
};

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

#[derive(Debug)]
pub struct FSTreeDiff {
    pub new_files: Vec<RSFile>,
    pub changed_files: Vec<RSFile>,
    pub removed_files: Vec<RSFile>,
}

impl FSTreeDiff {
    pub fn has_changes(&self) -> bool {
        (self.new_files.len() + self.changed_files.len() + self.removed_files.len()) > 0
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

    pub fn get_conflicting_files(&self, file_paths: Vec<String>) -> Vec<RSFile> {
        let conflicting_files = self
            .files
            .iter()
            .filter(|file| file_paths.contains(&file.path.to_string()))
            .cloned()
            .collect();
        conflicting_files
    }

    pub fn diff(&self, old_fs_tree: &Self) -> Result<FSTreeDiff> {
        let mut new_files = vec![];
        let mut changed_files = vec![];

        for file in self.files.clone() {
            let old_file = old_fs_tree
                .files
                .iter()
                .find(|old_file| old_file.path == file.path);

            if old_file.is_none() {
                new_files.push(file);
                continue;
            }
            let old_file = old_file.unwrap();
            if file.sha_256_digest != old_file.sha_256_digest {
                changed_files.push(file);
                continue;
            }
        }

        let removed_files = old_fs_tree
            .files
            .iter()
            .filter(|old_file| !self.files.iter().any(|file| old_file.path == file.path))
            .cloned()
            .collect();

        Ok(FSTreeDiff {
            new_files,
            changed_files,
            removed_files,
        })
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

    let mut builder_base = WalkBuilder::new(dir);
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
        if path.is_dir() && path != *dir && !is_rs_dir(&path, depth) {
            file_tree_items.extend(read_dir(&path, depth + 1, max_depth, root, ignores)?);
        } else if path.is_file() {
            let file_path = build_relative_file_path(&path, root);
            let sha256_digest: Sha256Digest = generate_sha256_digest(&path).unwrap();
            let size = std::fs::metadata(&path)?.len();
            let file = RSFile::new(file_path, sha256_digest, depth, size);
            file_tree_items.push(file);
        }
    }
    Ok(file_tree_items)
}

fn is_rs_dir(path: &Path, depth: u16) -> bool {
    return depth == 0 && path.file_name() == Some(OsStr::new(".rs"));
}

fn build_relative_file_path(path: &Path, root: &str) -> String {
    let suffix = match root.ends_with('/') {
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
    fn file_diffing() {
        let path = PathBuf::from_str("./test-data").unwrap();
        let old_fs_tree = FSTree::build(path, None).unwrap();
        let mut fs_tree = old_fs_tree.clone();
        let removed_file = fs_tree.files.pop().unwrap();
        println!("removed: {removed_file:?}");
        let mut changed_file = &mut fs_tree.files[0];
        changed_file.sha_256_digest =
            String::from("982bc87271bad526f4659eb12ecf1fd1295ae9fe0acfcfc83539fb9c0e523f5e");
        let changed_file = changed_file.clone();
        let new_file = RSFile::new(
            "new_file.elm".into(),
            "982bc87271bad526f4659eb12ecf1fd1295ae9fe0acfcfc83539fb9c0e523f5e".into(),
            0,
            123,
        );
        fs_tree.files.push(new_file.clone());

        println!("\n");
        println!("old:");
        println!("{old_fs_tree:?}");

        println!("\n");
        println!("new:");
        println!("{fs_tree:?}");

        let files_diff = fs_tree.diff(&old_fs_tree).unwrap();
        println!("\n");
        println!("diff:");
        println!("{files_diff:?}");

        assert_eq!(files_diff.new_files, vec![new_file]);
        assert_eq!(files_diff.changed_files, vec![changed_file]);
        assert_eq!(files_diff.removed_files, vec![removed_file]);
    }

    #[test]
    fn scans_a_directory_recursively() {
        let path = PathBuf::from_str("./test-data").unwrap();
        let mut fs_tree = FSTree::build(path.clone(), None).unwrap();
        let files = vec![
            RSFile::new(
                String::from("other_folder/other_file.hs"),
                String::from("982bc87271bad527f4659eb12ecf1fd1295ae9fe0acfcfc83539fb9c0e523f64"),
                1,
                53_u64,
            ),
            RSFile::new(
                String::from("hello.ex"),
                String::from("1d8326dc32bc35812503ecfcce8ca3db0f025fb84d589df4e687b96f6cdf03fe"),
                0,
                159_u64,
            ),
        ];
        let mut target_fs_tree = FSTree {
            files,
            root: path,
            max_depth: None,
        };
        target_fs_tree.files.sort();
        fs_tree.files.sort();

        assert_eq!(target_fs_tree.max_depth, fs_tree.max_depth);
        assert_eq!(target_fs_tree.root, fs_tree.root);
        assert_eq!(target_fs_tree, fs_tree);
    }
}
