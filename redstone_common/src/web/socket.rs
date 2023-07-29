use serde::de::DeserializeOwned;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};
use websocket::{
    sync::{stream::NetworkStream, Client},
    OwnedMessage,
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
        socket::{
            AbortMessage, CheckFileMessage, CommitMessage, DownloadChunkMessage,
            FinishDownloadMessage, SocketMessage, SocketOperation,
        },
        Result,
    },
};

pub fn send_message(
    client: &mut Client<Box<dyn NetworkStream + std::marker::Send>>,
    message: &OwnedMessage,
) -> Result<()> {
    Ok(client.send_message(message)?)
}

pub fn receive_message<T: DeserializeOwned>(
    client: &mut Client<Box<dyn NetworkStream + std::marker::Send>>,
) -> Result<T> {
    match client.recv_message()? {
        OwnedMessage::Text(str) => Ok(serde_json::from_str(&str)?),
        OwnedMessage::Binary(bin) => Ok(serde_json::from_slice(&bin)?),
        _ => unreachable!(),
    }
}

pub fn receive_raw_message(
    client: &mut Client<Box<dyn NetworkStream + std::marker::Send>>,
) -> Result<OwnedMessage> {
    Ok(client.recv_message()?)
}

pub struct AbortUpdateMessageFactory {
    upload_token: String,
}

impl AbortUpdateMessageFactory {
    pub fn new(upload_token: String) -> Self {
        Self { upload_token }
    }
}

impl SocketMessage for AbortUpdateMessageFactory {
    const OPERATION: SocketOperation = SocketOperation::Abort;
    fn get_payload(&mut self) -> Result<OwnedMessage> {
        let message = AbortMessage {
            upload_token: self.upload_token.to_string(),
            operation: Self::OPERATION,
        };

        Ok(OwnedMessage::Text(serde_json::to_string(&message)?))
    }
}

// pub struct DeclareFileMessageFactory {
//     upload_token: String,
//     file_id: String,
// }

// impl DeclareFileMessageFactory {
//     pub fn new(upload_token: &String, file: &api::File) -> Self {
//         Self {
//             upload_token: upload_token.to_owned(),
//             file_id: file.id.to_owned(),
//         }
//     }
// }

pub struct FileUploadMessageFactory {
    file_path: PathBuf,
    chunk_offset: usize,
    file_size: usize,
    read_bytes: usize,
    pub last_chunk_size: usize,
    times_sent: usize,
}

impl FileUploadMessageFactory {
    pub fn new(file: &api::File, root_folder: PathBuf) -> Self {
        let file_path = root_folder.join(file.path.clone());
        let file_size = std::fs::metadata(&file_path).unwrap().len();
        Self {
            file_path,
            chunk_offset: 0,
            file_size: file_size as usize,
            read_bytes: 0,
            last_chunk_size: 0,
            times_sent: 0,
        }
    }

    pub fn has_data_to_fetch(&self) -> bool {
        self.remaining_bytes_to_read() > 0 || (self.file_size == 0 && self.times_sent == 0)
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
        self.times_sent += 1;
        self.read_bytes += chunk_size;
        Ok(buffer)
    }
}

impl SocketMessage for FileUploadMessageFactory {
    const OPERATION: SocketOperation = SocketOperation::UploadChunk;

    fn get_payload(&mut self) -> Result<OwnedMessage> {
        Ok(OwnedMessage::Binary(self.get_next_chunk()?))
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

impl SocketMessage for CommitMessageFactory {
    const OPERATION: SocketOperation = SocketOperation::Commit;

    fn get_payload(&mut self) -> Result<OwnedMessage> {
        let message = CommitMessage {
            upload_token: self.upload_token.to_string(),
            operation: Self::OPERATION,
        };
        Ok(OwnedMessage::Text(serde_json::to_string(&message)?))
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

impl SocketMessage for CheckFileMessageFactory {
    const OPERATION: SocketOperation = SocketOperation::CheckFile;
    fn get_payload(&mut self) -> Result<OwnedMessage> {
        let message = CheckFileMessage {
            upload_token: self.upload_token.to_string(),
            file_id: self.file_id.to_string(),
            operation: Self::OPERATION,
        };

        Ok(OwnedMessage::Text(serde_json::to_string(&message)?))
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

impl SocketMessage for DownloadChunkMessageFactory {
    const OPERATION: SocketOperation = SocketOperation::DownloadChunk;
    fn get_payload(&mut self) -> Result<OwnedMessage> {
        let message = DownloadChunkMessage {
            operation: Self::OPERATION,
            download_token: self.download_token.to_string(),
            file_id: self.file_id.to_string(),
            byte_limit: TCP_FILE_CHUNK_SIZE,
            offset: self.offset,
        };
        self.offset += 1;
        Ok(OwnedMessage::Text(serde_json::to_string(&message)?))
    }
}

pub struct FinishDownloadMessageFactory {
    pub download_token: String,
}

impl FinishDownloadMessageFactory {
    pub fn new(download_token: String) -> Self {
        Self { download_token }
    }
}

impl SocketMessage for FinishDownloadMessageFactory {
    const OPERATION: SocketOperation = SocketOperation::FinishDownload;
    fn get_payload(&mut self) -> Result<OwnedMessage> {
        let message = FinishDownloadMessage {
            download_token: self.download_token.to_string(),
            operation: Self::OPERATION,
        };
        Ok(OwnedMessage::Text(serde_json::to_string(&message)?))
    }
}
