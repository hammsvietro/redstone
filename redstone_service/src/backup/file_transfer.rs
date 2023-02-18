use std::{
    borrow::BorrowMut,
    path::{Path, PathBuf},
};

use redstone_common::{
    constants::TCP_FILE_CHUNK_SIZE,
    model::{
        api::File as RSFile,
        tcp::{TcpMessage, TcpMessageResponse, TcpMessageResponseStatus},
        RedstoneError, Result,
    },
    web::tcp::{
        receive_message, send_message, CheckFileMessageFactory, CommitMessageFactory,
        DownloadChunkMessageFactory, FileUploadMessageFactory, FinishDownloadMessageFactory,
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
    println!("will send files");
    let stream = TcpStream::connect("127.0.0.1:8000").await?;
    let mut stream = BufReader::new(stream);
    let mut bytes_sent: u64 = 0;
    for file in files {
        send_file(
            &mut stream,
            file,
            upload_token,
            &root_folder,
            &progress_emitter,
            &mut bytes_sent,
        )
        .await?;
    }
    send_commit_msg(&mut stream, upload_token).await
}

pub async fn download_files(
    root: PathBuf,
    files: &Vec<RSFile>,
    download_token: String,
) -> Result<()> {
    let stream = TcpStream::connect("127.0.0.1:8000").await?;
    let mut stream = BufReader::new(stream);
    let mut _bytes_received: u64 = 0;
    for file in files {
        download_file(
            &mut stream,
            file,
            &root,
            download_token.clone(),
            &mut _bytes_received,
        )
        .await?;
        println!("downloaded {}", file.path);
    }
    let packet = FinishDownloadMessageFactory::new(download_token.to_string()).get_tcp_payload()?;
    send_message(&mut stream, &packet).await?;
    let response: TcpMessageResponse<Vec<u8>> = receive_message(&mut stream).await?;
    if response.status != TcpMessageResponseStatus::Ok {
        // TODO: send abort message
        let error = format!(
            "Error commiting finalizing download.\nServer responded: {}",
            response.reason.unwrap()
        );
        return Err(RedstoneError::BaseError(error));
    }
    Ok(())
}

async fn send_file(
    stream: &mut BufReader<TcpStream>,
    file: &RSFile,
    upload_token: &String,
    root_folder: &Path,
    progress_emitter: &UnboundedSender<u64>,
    bytes_sent: &mut u64,
) -> Result<()> {
    println!("Uploading {} file", file.path);
    let mut retry_count: u8 = 0;
    loop {
        let mut file_upload_message =
            FileUploadMessageFactory::new(upload_token, file, root_folder.to_path_buf());
        while file_upload_message.has_data_to_fetch() {
            let packet = file_upload_message.get_tcp_payload()?;
            send_message(stream.borrow_mut(), &packet).await?;
            *bytes_sent += file_upload_message.last_chunk_size as u64;
            progress_emitter.send(*bytes_sent)?;
            let response: TcpMessageResponse<()> = receive_message(stream.borrow_mut()).await?;
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
        send_message(stream.borrow_mut(), &check_file_message).await?;
        let response: TcpMessageResponse<()> = receive_message(stream.borrow_mut()).await?;
        if response.status == TcpMessageResponseStatus::Ok {
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
    Ok(())
}

async fn send_commit_msg(stream: &mut BufReader<TcpStream>, upload_token: &str) -> Result<()> {
    let commit_payload = CommitMessageFactory::new(upload_token.to_owned()).get_tcp_payload()?;
    println!("Sending commit msg!");
    send_message(stream.borrow_mut(), &commit_payload).await?;
    let response: TcpMessageResponse<()> = receive_message(stream.borrow_mut()).await?;
    if response.status != TcpMessageResponseStatus::Ok {
        let error = format!(
            "Error commiting backup transaction.\nServer responded: {}",
            response.reason.unwrap()
        );
        return Err(RedstoneError::BaseError(error));
    }
    Ok(())
}

async fn download_file(
    stream: &mut BufReader<TcpStream>,
    file: &RSFile,
    root: &Path,
    download_token: String,
    _bytes_received: &mut u64,
) -> Result<()> {
    let mut path = root.to_path_buf();
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
        send_message(stream.borrow_mut(), &packet).await?;
        let response: TcpMessageResponse<Vec<u8>> = receive_message(stream.borrow_mut()).await?;
        if response.data.is_none() {
            break;
        }
        let data = response.data.unwrap();
        *_bytes_received += data.len() as u64;
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
    Ok(())
}
