/// Socket message models
use super::Result;
use serde::{Deserialize, Serialize};
use websocket::OwnedMessage;

pub trait SocketMessage {
    const OPERATION: SocketOperation;
    fn get_payload(&mut self) -> Result<OwnedMessage>;
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SocketOperation {
    Abort,
    UploadChunk,
    Commit,
    CheckFile,
    DownloadChunk,
    FinishDownload,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AbortMessage {
    pub upload_token: String,
    pub operation: SocketOperation,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DownloadChunkMessage {
    pub download_token: String,
    pub operation: SocketOperation,
    pub file_id: String,
    pub offset: usize,
    pub byte_limit: usize,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SocketMessageResponseStatus {
    Ok,
    Error,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SocketMessageResponse<T> {
    pub status: SocketMessageResponseStatus,
    pub data: Option<T>,
    pub reason: Option<String>,
    pub retry: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FileUploadMessage {
    pub upload_token: String,
    pub operation: SocketOperation,
    pub file_id: String,
    pub file_size: usize,
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
    pub last_chunk: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CommitMessage {
    pub upload_token: String,
    pub operation: SocketOperation,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FinishDownloadMessage {
    pub download_token: String,
    pub operation: SocketOperation,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CheckFileMessage {
    pub upload_token: String,
    pub file_id: String,
    pub operation: SocketOperation,
}
