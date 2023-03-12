use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::web::api::get_api_base_url;

use super::fs_tree::{FSTreeDiff, RSFile};

pub enum Endpoints {
    Clone,
    Declare,
    FetchUpdate(String), // backup_id
    Login,
    Push,
    Pull,
}

impl Endpoints {
    pub fn get_url(&self) -> Url {
        let base_url = get_api_base_url();
        let sufix: String = match self {
            Self::Login => "/api/login".to_owned(),

            Self::Clone => "/api/download/clone".to_owned(),
            Self::Pull => "/api/download/pull".to_owned(),

            Self::Declare => "/api/upload/declare".to_owned(),
            Self::Push => "/api/upload/push".to_owned(),

            Self::FetchUpdate(backup_id) => format!("/api/update/fetch/{}", backup_id.to_owned()),
        };
        base_url.join(&sufix).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub enum FileOperation {
    Add,
    Update,
    Remove,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct FileUploadRequest {
    pub path: String,
    pub sha_256_digest: Option<String>,
    pub operation: FileOperation,
    pub size: u64,
}

impl FileUploadRequest {
    pub fn new(
        path: String,
        sha_256_digest: Option<String>,
        operation: FileOperation,
        size: u64,
    ) -> Self {
        Self {
            path,
            sha_256_digest,
            operation,
            size,
        }
    }

    pub fn from_diff(diff: &FSTreeDiff) -> Vec<Self> {
        let new_files: Vec<Self> = diff
            .new_files
            .iter()
            .clone()
            .map(|f| Self {
                path: f.path.to_owned(),
                sha_256_digest: Some(f.sha_256_digest.to_owned()),
                operation: FileOperation::Add,
                size: f.size,
            })
            .collect();

        let changed_files: Vec<Self> = diff
            .changed_files
            .iter()
            .clone()
            .map(|f| Self {
                path: f.path.to_owned(),
                sha_256_digest: Some(f.sha_256_digest.to_owned()),
                operation: FileOperation::Update,
                size: f.size,
            })
            .collect();

        let removed_files: Vec<Self> = diff
            .removed_files
            .iter()
            .clone()
            .map(|f| Self {
                path: f.path.to_owned(),
                sha_256_digest: None,
                operation: FileOperation::Remove,
                size: f.size,
            })
            .collect();

        [new_files, removed_files, changed_files].concat()
    }
}

impl From<RSFile> for FileUploadRequest {
    fn from(rs_file: RSFile) -> Self {
        Self {
            path: rs_file.path,
            sha_256_digest: Some(rs_file.sha_256_digest),
            operation: FileOperation::Add,
            size: rs_file.size,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum ApiRequestWrapper<T: Serialize> {
    Ok(T),
    Err,
}

#[derive(Deserialize, Serialize)]
pub struct DeclareBackupRequest<'a> {
    pub files: Vec<FileUploadRequest>,
    pub root: PathBuf,
    pub name: &'a str,
}

impl<'a> DeclareBackupRequest<'a> {
    pub fn new(name: &'a str, root: PathBuf, files: Vec<FileUploadRequest>) -> Self {
        Self { name, root, files }
    }
}

#[derive(Deserialize, Serialize)]
pub struct CloneRequest {
    pub backup_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DownloadResponse {
    pub backup: Backup,
    pub files: Vec<File>,
    pub download_token: String,
    pub update: Update,
    pub total_bytes: usize,
}

impl CloneRequest {
    pub fn new(backup_name: String) -> Self {
        Self { backup_name }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UploadResponse {
    pub backup: Backup,
    pub files: Vec<File>,
    pub update: Update,
    pub upload_token: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PushRequest {
    pub backup_id: String,
    pub files: Vec<FileUploadRequest>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PullRequest {
    pub backup_id: String,
    pub update_id: String,
}

/* SERVER ENTITIES */

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Backup {
    pub id: String,
    pub name: String,
    pub entrypoint: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct File {
    pub id: String,
    pub path: String,
    pub sha256_checksum: String,
    pub last_update: FileUpdate,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FileUpdate {
    pub operation: FileOperation,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Update {
    pub id: String,
    pub hash: String,
    pub message: String,
}
