use std::path::PathBuf;

use redstone_common::{
    constants::TCP_FILE_CHUNK_SIZE,
    model::{
        api::File as RSFile,
        tcp::{TcpMessage, TcpMessageResponse, TcpMessageResponseStatus},
        RedstoneError, Result,
    },
    web::tcp::{
        receive_message, send_message, CheckFileMessageFactory, CommitMessageFactory,
        DownloadChunkMessageFactory, FileUploadMessageFactory,
    },
};

use tokio::{
    io::{AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::mpsc::UnboundedSender,
};

pub async fn send_files(
    files: &Vec<RSFile>,
    upload_token: &String,
    root_folder: PathBuf,
    progress_emitter: UnboundedSender<u64>,
) -> Result<()> {
    let stream = TcpStream::connect("127.0.0.1:8000").await?;
    let mut stream = BufReader::new(stream);
    let mut bytes_sent: u64 = 0;
    for file in files {
        println!("Uploading {} file", file.path);
        let mut retry_count: u8 = 0;
        loop {
            let mut file_upload_message =
                FileUploadMessageFactory::new(upload_token, file, root_folder.clone());
            while file_upload_message.has_data_to_fetch() {
                let packet = file_upload_message.get_tcp_payload()?;
                send_message(&mut stream, &packet).await?;
                bytes_sent += file_upload_message.last_chunk_size as u64;
                progress_emitter.send(bytes_sent)?;
                let response: TcpMessageResponse<()> = receive_message(&mut stream).await?;
                if response.status != TcpMessageResponseStatus::Ok {
                    // TODO: send abort message
                    let error = format!(
                        "Error commiting backup transaction.\nServer responded: {}",
                        response.reason.unwrap()
                    );
                    return Err(RedstoneError::BaseError(error));
                }
            }
            let check_file_message =
                CheckFileMessageFactory::new(upload_token, &file.id).get_tcp_payload()?;
            println!("Verifying checksum");
            send_message(&mut stream, &check_file_message).await?;
            let response: TcpMessageResponse<()> = receive_message(&mut stream).await?;
            if response.status == TcpMessageResponseStatus::Ok {
                println!("Checksum matches");
                println!("{} upload complete\n", file.path);
                break;
            } else {
                retry_count += 1;
                if retry_count > 4 {
                    return Err(RedstoneError::BaseError(format!(
                        "File upload retry count exceeded.\nServer returned: {:?}",
                        response.reason.unwrap()
                    )));
                }
            }
        }
    }
    let commit_payload = CommitMessageFactory::new(upload_token.clone()).get_tcp_payload()?;
    println!("Sending commit msg!");
    send_message(&mut stream, &commit_payload).await?;
    let response: TcpMessageResponse<()> = receive_message(&mut stream).await?;
    if response.status != TcpMessageResponseStatus::Ok {
        let error = format!(
            "Error commiting backup transaction.\nServer responded: {}",
            response.reason.unwrap()
        );
        return Err(RedstoneError::BaseError(error));
    }
    Ok(())
}

pub async fn download_files(
    root: PathBuf,
    files: &Vec<RSFile>,
    download_token: String,
) -> Result<()> {
    let stream = TcpStream::connect("127.0.0.1:8000").await?;
    let mut stream = BufReader::new(stream);
    let mut bytes_received: u64 = 0;
    for file in files {
        let mut path = root.clone();
        path.push(file.path.clone());
        if path.is_file() {
            tokio::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&path)
                .await?;
        } else if let Some(prefix) = path.parent() {
            tokio::fs::create_dir_all(prefix).await?;
        }
        let mut factory = DownloadChunkMessageFactory::new(download_token.clone(), file.id.clone());
        loop {
            let packet = factory.get_tcp_payload()?;
            send_message(&mut stream, &packet).await?;
            let response: TcpMessageResponse<Vec<u8>> = receive_message(&mut stream).await?;
            if response.data.is_none() {
                break;
            }
            let data = response.data.unwrap();
            bytes_received += data.len() as u64;
            let mut file = tokio::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(&path)
                .await?;
            file.write_all(&data).await?;
            if data.len() == TCP_FILE_CHUNK_SIZE {
                break;
            }
        }
        println!("downloaded {}", file.path);
    }
    Ok(())
}
