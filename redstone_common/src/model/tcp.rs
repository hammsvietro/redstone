/// TCP message models
use super::Result;
use serde::{Deserialize, Serialize};

pub trait TcpMessage {
    const OPERATION: TcpOperation;
    fn get_tcp_payload(&mut self) -> Result<Vec<u8>>;
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum TcpOperation {
    Abort,
    UploadChunk,
    Commit,
    CheckFile,
    DownloadChunk,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AbortMessage {
    pub upload_token: String,
    pub operation: TcpOperation,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DownloadChunkMessage {
    pub download_token: String,
    pub operation: TcpOperation,
    pub file_id: String,
    pub offset: usize,
    pub byte_limit: usize,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TcpMessageResponseStatus {
    Ok,
    Error,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TcpMessageResponse<T> {
    pub status: TcpMessageResponseStatus,
    pub data: Option<T>,
    pub reason: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FileUploadMessage {
    pub upload_token: String,
    pub operation: TcpOperation,
    pub file_id: String,
    pub file_size: usize,
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
    pub last_chunk: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CommitMessage {
    pub upload_token: String,
    pub operation: TcpOperation,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CheckFileMessage {
    pub upload_token: String,
    pub file_id: String,
    pub operation: TcpOperation,
}
