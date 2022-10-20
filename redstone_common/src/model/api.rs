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

#[derive(Deserialize, Serialize, Debug)]
pub struct DeclareBackupResponse {
    pub backup: Backup
}

/* SERVER ENTITIES */

#[derive(Deserialize, Serialize, Debug)]
pub struct Backup {
    pub id: String,
    pub name: String,
    pub entrypoint: String,
    pub files: Vec<File>
}

#[derive(Deserialize, Serialize, Debug)]
pub struct File {
    pub id: String,
    pub path: String,
    pub sha1_checksum: String
}
