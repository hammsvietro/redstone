use std::path::PathBuf;

use super::{fs_tree::FSTree, track::TrackRequest};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    server_url: String,
    auth_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexFile {
    pub config: BackupConfig,
    pub last_synced_fstree: FSTree,
}

impl IndexFile {
    pub fn new(track_req: TrackRequest, fs_tree: &FSTree) -> Self {
        Self {
            last_synced_fstree: fs_tree.to_owned(),
            config: BackupConfig {
                sync_every: track_req.sync_every,
                watch: track_req.watch,
                base_path: track_req.base_path,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupConfig {
    pub sync_every: Option<String>,
    pub watch: bool,
    pub base_path: PathBuf,
}
