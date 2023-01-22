use std::path::PathBuf;

use super::{
    api::{Backup, DeclareBackupResponse, Update},
    ipc::track::TrackRequest,
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
    pub current_update: Update,
}

impl IndexFile {
    pub fn new(declare_response: DeclareBackupResponse, track_request: &TrackRequest) -> Self {
        Self {
            backup: declare_response.backup,
            config: BackupConfig {
                sync_every: track_request.sync_every.clone(),
                watch: track_request.watch,
                entrypoint: track_request.base_path.to_str().unwrap().into(),
            },
            current_update: declare_response.update,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupConfig {
    pub sync_every: Option<String>,
    pub watch: bool,
    pub entrypoint: String,
}

pub fn get_index_file_for_path(path: &PathBuf) -> PathBuf {
    let mut path = path.clone();
    path.push(".rs");
    path.push("index");
    path
}
