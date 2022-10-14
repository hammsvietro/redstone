use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::fs_tree::RSFile;

#[derive(Deserialize, Serialize)]
pub struct DeclareBackupRequest {
    pub files: Vec<RSFile>,
    pub root: PathBuf,
    pub name: String,
}

impl DeclareBackupRequest {
    pub fn new(name: String, root: PathBuf, files: Vec<RSFile>) -> Self {
        Self { name, root, files }
    }
}
