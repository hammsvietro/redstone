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

impl From<TrackRequest> for IndexFile {
    fn from(req: TrackRequest) -> Self {
        Self {
            last_synced_fstree: req.fs_tree,
            config: BackupConfig {
                sync_every: req.sync_every,
                watch: req.watch,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupConfig {
    pub sync_every: Option<String>,
    pub watch: bool,
}
