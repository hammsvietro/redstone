use super::Result;
use crate::constants::TCP_FILE_CHUNK_SIZE;
use serde::{Deserialize, Serialize};
/// TCP message models
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
};

pub struct AbortUpdateMessageFactory {
    upload_token: String,
}

impl AbortUpdateMessageFactory {
    pub fn new(upload_token: String) -> Self {
        Self { upload_token }
    }
}

impl TcpMessage for AbortUpdateMessageFactory {
    const OPERATION: TcpOperation = TcpOperation::Abort;
    fn get_tcp_payload(&mut self) -> Result<Vec<u8>> {
        let message = AbortMessage {
            upload_token: self.upload_token.to_string(),
            operation: Self::OPERATION,
        };
        Ok(bson::to_vec(&message)?)
    }
}

pub struct FileUploadMessageFactory {
    upload_token: String,
    file_id: String,
    file_path: PathBuf,
    chunk_offset: usize,
    file_size: usize,
    read_bytes: usize,
}

impl FileUploadMessageFactory {
    pub fn new(upload_token: String, file: super::api::File, root_folder: PathBuf) -> Self {
        let file_path = root_folder.join(file.path);
        let file_size = std::fs::metadata(&file_path).unwrap().len();
        Self {
            upload_token,
            file_id: file.id,
            file_path,
            chunk_offset: 0,
            file_size: file_size as usize,
            read_bytes: 0,
        }
    }

    pub fn has_data_to_fetch(&self) -> bool {
        self.remaining_bytes_to_read() > 0
    }

    fn remaining_bytes_to_read(&self) -> usize {
        isize::max((self.file_size - self.read_bytes) as isize, 0) as usize
    }

    fn get_next_chunk(&mut self) -> Result<Vec<u8>> {
        let chunk_size = usize::min(self.remaining_bytes_to_read(), TCP_FILE_CHUNK_SIZE);
        let mut file = File::open(&self.file_path)?;
        file.seek(SeekFrom::Start(self.read_bytes as u64))?;
        let mut buffer: Vec<u8> = vec![0; chunk_size];
        file.read_exact(&mut buffer)?;
        self.chunk_offset += 1;
        self.read_bytes += chunk_size;
        Ok(buffer)
    }
}

impl TcpMessage for FileUploadMessageFactory {
    const OPERATION: TcpOperation = TcpOperation::UploadChunk;

    fn get_tcp_payload(&mut self) -> Result<Vec<u8>> {
        let data = self.get_next_chunk()?;
        let message = FileUploadMessage {
            upload_token: self.upload_token.to_string(),
            operation: TcpOperation::UploadChunk,
            file_id: self.file_id.to_string(),
            file_size: self.file_size,
            data,
            last_chunk: !self.has_data_to_fetch(),
        };
        let encoded = bson::to_vec(&message)?;
        Ok(encoded)
    }
}

pub trait TcpMessage {
    const OPERATION: TcpOperation;
    fn get_tcp_payload(&mut self) -> Result<Vec<u8>>;
}

#[derive(Deserialize, Serialize, Debug)]
pub enum TcpOperation {
    Abort,
    UploadChunk,
}

#[derive(Deserialize, Serialize, Debug)]
struct AbortMessage {
    pub upload_token: String,
    pub operation: TcpOperation,
}

#[derive(Deserialize, Serialize, Debug)]
struct FileUploadMessage {
    pub upload_token: String,
    pub operation: TcpOperation,
    pub file_id: String,
    pub file_size: usize,
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
    pub last_chunk: bool,
}
