use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use super::fs_tree::FSTreeItem;

#[derive(Deserialize, Serialize)]
pub struct DeclareBackupRequest {
    pub files: Vec<FSTreeItem>,
    pub root: PathBuf,
    pub name: String,
}

impl DeclareBackupRequest {
    pub fn new(name: String, root: PathBuf, files: Vec<FSTreeItem>) -> Self {
        Self { name, root, files }
    }
}
