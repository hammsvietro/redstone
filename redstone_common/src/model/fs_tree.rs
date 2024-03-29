use colored::Colorize;
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr,
    fmt::Debug,
    path::{Path, PathBuf},
};

use crate::util::generate_sha256_digest;

use super::{
    ipc::{FileAction, FileActionProgress},
    ArgumentError, RedstoneError, Result,
};

type Sha256Digest = String;

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RSFile {
    pub path: String,
    pub sha_256_digest: String,
    pub size: u64,
}

impl RSFile {
    pub fn new(path: String, sha_256_digest: String, size: u64) -> Self {
        Self {
            path,
            sha_256_digest,
            size,
        }
    }
}

#[derive(Debug, Default)]
pub struct FSTreeDiff {
    pub new_files: Vec<RSFile>,
    pub changed_files: Vec<RSFile>,
    pub removed_files: Vec<RSFile>,
}

impl FSTreeDiff {
    pub fn has_changes(&self) -> bool {
        (self.new_files.len() + self.changed_files.len() + self.removed_files.len()) > 0
    }

    pub fn total_size(&self) -> u64 {
        [
            self.new_files.clone(),
            self.changed_files.clone(),
            self.removed_files.clone(),
        ]
        .concat()
        .iter()
        .map(|file| file.size)
        .sum()
    }

    pub fn get_changes_message(&self) -> String {
        let mut message = String::new();

        if !self.has_changes() {
            return String::from("\nNo changes.\n");
        }

        let new_files = self
            .new_files
            .iter()
            .cloned()
            .map(|f| f.path)
            .collect::<Vec<String>>();
        if !new_files.is_empty() {
            message += "\nAdded:\n";
            for path in new_files {
                message += &format!("{}\n", path.green());
            }
        }

        let updated_files = self
            .changed_files
            .iter()
            .cloned()
            .map(|f| f.path)
            .collect::<Vec<String>>();

        if !updated_files.is_empty() {
            message += "\nChanged:\n";
            for path in updated_files {
                message += &format!("{}\n", path.purple());
            }
        }
        let removed_files = self
            .removed_files
            .iter()
            .cloned()
            .map(|f| f.path)
            .collect::<Vec<String>>();
        if !removed_files.is_empty() {
            message += "\nRemoved:\n";
            for path in removed_files {
                message += &format!("{}\n", path.red());
            }
        }

        message
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct FSTree {
    pub files: Vec<RSFile>,
    pub root: PathBuf,
}

impl FSTree {
    pub fn build(
        root: PathBuf,
        progress_handler_fn: Option<&dyn Fn(FileActionProgress)>,
    ) -> Result<FSTree> {
        let mut fs_tree = Self::build_base(root)?;
        let files = read_dir(&fs_tree.root, 0, &mut Vec::new())?;
        fs_tree.files = build_rs_files(&fs_tree.root, files, progress_handler_fn)?;
        Ok(fs_tree)
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

    fn build_base(root: PathBuf) -> Result<FSTree> {
        let root_is_file = root.is_file();
        let fs_tree = FSTree {
            root,
            files: Vec::new(),
        };

        let root_as_string = &fs_tree.root.to_str().unwrap();
        if root_is_file {
            return Err(RedstoneError::ArgumentError(
                ArgumentError::PathCannotBeAFile(String::from(*root_as_string)),
            ));
        }
        Ok(fs_tree)
    }
}

fn read_dir(dir: &PathBuf, depth: u16, ignores: &mut Vec<String>) -> Result<Vec<PathBuf>> {
    let mut file_tree_items = Vec::new();
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
            file_tree_items.extend(read_dir(&path, depth + 1, ignores)?);
        } else if path.is_file() {
            file_tree_items.push(path);
        }
    }
    Ok(file_tree_items)
}

fn build_rs_files(
    root: &Path,
    files: Vec<PathBuf>,
    progress_handler_fn: Option<&dyn Fn(FileActionProgress)>,
) -> Result<Vec<RSFile>> {
    let files_count = files.len() as u64;
    let mut files_hashed = 0;
    files
        .iter()
        .map(|path| {
            let file_path = build_relative_file_path(path, root);
            let size = std::fs::metadata(path)?.len();

            if let Some(handler) = progress_handler_fn {
                handler(FileActionProgress {
                    total: files_count,
                    current_file_name: file_path.to_owned(),
                    progress: files_hashed,
                    operation: FileAction::Hash,
                });
            }

            let sha256_digest: Sha256Digest = generate_sha256_digest(path)?;
            files_hashed += 1;
            Ok(RSFile::new(file_path, sha256_digest, size))
        })
        .collect::<Result<_>>()
}

fn is_rs_dir(path: &Path, depth: u16) -> bool {
    return depth == 0 && path.file_name() == Some(OsStr::new(".rs"));
}

fn build_relative_file_path(path: &Path, root: &Path) -> String {
    let root = root.to_str().unwrap();
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
        let mut changed_file = &mut fs_tree.files[0];
        changed_file.sha_256_digest =
            String::from("982bc87271bad526f4659eb12ecf1fd1295ae9fe0acfcfc83539fb9c0e523f5e");
        let changed_file = changed_file.clone();
        let new_file = RSFile::new(
            "new_file.elm".into(),
            "982bc87271bad526f4659eb12ecf1fd1295ae9fe0acfcfc83539fb9c0e523f5e".into(),
            123,
        );
        fs_tree.files.push(new_file.clone());

        let files_diff = fs_tree.diff(&old_fs_tree).unwrap();

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
                53_u64,
            ),
            RSFile::new(
                String::from("hello.ex"),
                String::from("1d8326dc32bc35812503ecfcce8ca3db0f025fb84d589df4e687b96f6cdf03fe"),
                159_u64,
            ),
        ];
        let mut target_fs_tree = FSTree { files, root: path };
        target_fs_tree.files.sort();
        fs_tree.files.sort();

        assert_eq!(target_fs_tree.root, fs_tree.root);
        assert_eq!(target_fs_tree, fs_tree);
    }
}
