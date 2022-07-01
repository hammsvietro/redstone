use serde::{Deserialize, Serialize};

use super::fs_tree::FSTree;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrackRequest {
    pub fs_tree: FSTree,
    pub detatched: bool,
    pub sync_every: Option<String>,
    pub watch: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackMessageResponse {
    pub data: String,
}
