use serde::de::DeserializeOwned;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
};

use crate::{
    constants::TCP_FILE_CHUNK_SIZE,
    model::{
        api,
        tcp::{
            AbortMessage, CheckFileMessage, CommitMessage, DownloadChunkMessage, FileUploadMessage,
            TcpMessage, TcpOperation,
        },
        Result,
    },
};

pub async fn send_message(stream: &mut BufReader<TcpStream>, packet: &[u8]) -> Result<()> {
    let packet_size = get_message_size_in_bytes(packet);
    Ok(stream.write_all(&[&packet_size, packet].concat()).await?)
}

pub async fn receive_message<T: DeserializeOwned>(stream: &mut BufReader<TcpStream>) -> Result<T> {
    let mut incoming_packet_buf: [u8; 4] = [0; 4];
    stream.read_exact(&mut incoming_packet_buf).await?;
    let incoming_packet_size = u32::from_be_bytes(incoming_packet_buf);
    let mut buffer = vec![0; incoming_packet_size as usize];
    stream.read_exact(&mut buffer).await?;
    Ok(bson::from_slice(&buffer)?)
}

fn get_message_size_in_bytes(message: &[u8]) -> [u8; 4] {
    (message.len() as u32).to_be_bytes()
}

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
    pub last_chunk_size: usize,
}

impl FileUploadMessageFactory {
    pub fn new(upload_token: &String, file: &api::File, root_folder: PathBuf) -> Self {
        let file_path = root_folder.join(file.path.clone());
        let file_size = std::fs::metadata(&file_path).unwrap().len();
        Self {
            upload_token: upload_token.to_owned(),
            file_id: file.id.to_string(),
            file_path,
            chunk_offset: 0,
            file_size: file_size as usize,
            read_bytes: 0,
            last_chunk_size: 0,
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
        self.last_chunk_size = chunk_size;
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

pub struct CommitMessageFactory {
    pub upload_token: String,
}

impl CommitMessageFactory {
    pub fn new(upload_token: String) -> Self {
        CommitMessageFactory { upload_token }
    }
}

impl TcpMessage for CommitMessageFactory {
    const OPERATION: TcpOperation = TcpOperation::Commit;

    fn get_tcp_payload(&mut self) -> Result<Vec<u8>> {
        let message = CommitMessage {
            upload_token: self.upload_token.to_string(),
            operation: Self::OPERATION,
        };
        Ok(bson::to_vec(&message)?)
    }
}

pub struct CheckFileMessageFactory {
    pub upload_token: String,
    pub file_id: String,
}

impl CheckFileMessageFactory {
    pub fn new(upload_token: &String, file_id: &String) -> Self {
        Self {
            upload_token: upload_token.to_owned(),
            file_id: file_id.to_owned(),
        }
    }
}

impl TcpMessage for CheckFileMessageFactory {
    const OPERATION: TcpOperation = TcpOperation::CheckFile;
    fn get_tcp_payload(&mut self) -> Result<Vec<u8>> {
        let message = CheckFileMessage {
            upload_token: self.upload_token.to_string(),
            file_id: self.file_id.to_string(),
            operation: Self::OPERATION,
        };

        Ok(bson::to_vec(&message)?)
    }
}

pub struct DownloadChunkMessageFactory {
    pub download_token: String,
    pub file_id: String,
    pub offset: usize,
}

impl DownloadChunkMessageFactory {
    pub fn new(download_token: String, file_id: String) -> Self {
        Self {
            download_token,
            file_id,
            offset: 0,
        }
    }
}

impl TcpMessage for DownloadChunkMessageFactory {
    const OPERATION: TcpOperation = TcpOperation::DownloadChunk;
    fn get_tcp_payload(&mut self) -> Result<Vec<u8>> {
        let message = DownloadChunkMessage {
            operation: Self::OPERATION,
            download_token: self.download_token.to_string(),
            file_id: self.file_id.to_string(),
            byte_limit: TCP_FILE_CHUNK_SIZE,
            offset: self.offset,
        };
        self.offset += 1;
        Ok(bson::to_vec(&message)?)
    }
}
