use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CloneRequest {
    pub path: PathBuf,
    pub backup_name: String,
}
