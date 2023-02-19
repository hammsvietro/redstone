use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrackRequest {
    pub base_path: PathBuf,
    pub name: String,
    pub detatched: bool,
    pub sync_every: Option<String>,
    pub watch: bool,
}
