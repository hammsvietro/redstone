use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};

use crate::model::{DomainError, RedstoneError};

use super::{
    api::{Backup, Update},
    fs_tree::FSTree,
    Result,
};
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
    pub last_fs_tree: FSTree,
    pub current_update: Update,
    pub latest_update: Update,
}

impl IndexFile {
    pub fn new(
        backup: Backup,
        current_update: Update,
        latest_update: Update,
        config: BackupConfig,
        fs_tree: FSTree,
    ) -> Self {
        Self {
            backup,
            current_update,
            latest_update,
            config,
            last_fs_tree: fs_tree,
        }
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        let metadata = std::fs::metadata(path)?;
        let mut buffer = vec![0_u8; metadata.len() as usize];
        let mut file = std::fs::File::open(path)?;
        file.read_exact(&mut buffer)?;
        if buffer.is_empty() {
            return Err(RedstoneError::DomainError(DomainError::BackupDoesntExist(
                path.to_str().unwrap().into(),
            )));
        }
        Ok(bincode::deserialize(&buffer)?)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;

        Ok(file.write_all(&bincode::serialize(self)?)?)
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
