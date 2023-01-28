use std::path::{Path, PathBuf};

use super::api::{Backup, Update};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    server_url: String,
    auth_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexFile {
    pub config: BackupConfig,
    pub backup: Backup,
    pub current_update: Update,
    pub latest_update: Update,
}

impl IndexFile {
    pub fn new(
        backup: Backup,
        current_update: Update,
        latest_update: Update,
        config: BackupConfig,
    ) -> Self {
        Self {
            backup,
            current_update,
            latest_update,
            config,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupConfig {
    pub sync_every: Option<String>,
    pub watch: bool,
}

impl BackupConfig {
    pub fn new(sync_every: Option<String>, watch: bool) -> Self {
        Self { sync_every, watch }
    }
}

pub fn get_index_file_for_path(path: &Path) -> PathBuf {
    let mut path = path.to_path_buf();
    path.push(".rs");
    path.push("index");
    path
}
