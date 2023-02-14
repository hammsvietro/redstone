use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::web::api::get_api_base_url;

use super::fs_tree::RSFile;

pub enum Endpoints {
    Clone,
    Declare,
    Login,
    Push,
}

impl Endpoints {
    pub fn get_url(&self) -> Url {
        let base_url = get_api_base_url();
        let sufix = match *self {
            Self::Clone => "/api/download/clone",
            Self::Declare => "/api/upload/declare",
            Self::Login => "/api/login",
            Self::Push => "/api/push",
        };
        base_url.join(sufix).unwrap()
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
pub struct CloneResponse {
    pub backup: Backup,
    pub files_to_download: Vec<File>,
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
pub struct DeclareBackupResponse {
    pub backup: Backup,
    pub files: Vec<File>,
    pub update: Update,
    pub upload_token: String,
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
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Update {
    hash: String,
    message: String,
}
