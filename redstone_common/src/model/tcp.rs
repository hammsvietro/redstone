/// TCP message models

use std::{path::PathBuf, fs::File, io::{Seek, SeekFrom, Read}};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::constants::TCP_FILE_CHUNK_SIZE;

use super::Result;

#[derive(Deserialize, Serialize)]
pub struct AbortUpdate {
    upload_token: String,

}

impl AbortUpdate {
    pub fn new(upload_token: String) -> Self {
        Self { upload_token }
    }
}

impl TcpMessage for AbortUpdate {
    const OPERATION: &'static str = "ABORT";
    fn get_tcp_payload(&mut self) -> Result<Vec<u8>> {
        let payload = json!({
            "upload_token": self.upload_token,
            "operation": Self::OPERATION
        });
        Ok(serde_json::to_vec(&payload)?)
    } 
}

#[derive(Deserialize, Serialize)]
pub struct FileUploadMessage {
    upload_token: String,
    file_id: String,
    file_path: PathBuf,
    chunk_offset: usize,
    file_size: usize,
    read_bytes: usize
}

impl FileUploadMessage {
    pub fn new(upload_token: String, file: super::api::File, root_folder: PathBuf) -> Self {
        let file_path = root_folder.join(file.path);
        let file_size = std::fs::metadata(&file_path).unwrap().len();
        Self {
            upload_token,
            file_id: file.id,
            file_path,
            chunk_offset: 0,
            file_size: file_size as usize,
            read_bytes: 0
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

impl TcpMessage for FileUploadMessage {
    const OPERATION: &'static str = "UPLOAD_CHUNK";

    fn get_tcp_payload(&mut self) -> Result<Vec<u8>> {
        let data = self.get_next_chunk()?;
        let payload = json!({
            "upload_token": self.upload_token,
            "operation": Self::OPERATION,
            "file_id": self.file_id,
            "file_size": self.file_size,
            "data": data,
            "last_chunk": !self.has_data_to_fetch()
        });
        Ok(serde_json::to_vec(&payload)?)
    } 
}

pub trait TcpMessage {
    const OPERATION: &'static str;
    fn get_tcp_payload(&mut self) -> Result<Vec<u8>>;
}
