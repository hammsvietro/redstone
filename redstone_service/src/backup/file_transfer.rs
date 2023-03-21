use std::{
    borrow::BorrowMut,
    path::{Path, PathBuf},
    time::Duration,
};

use async_recursion::async_recursion;
use redstone_common::{
    constants::TCP_FILE_CHUNK_SIZE,
    model::{
        api::{File as RSFile, FileOperation},
        ipc::{FileAction, FileActionProgress},
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
    time::sleep,
};

pub async fn send_files(
    files: &[RSFile],
    upload_token: &String,
    root_folder: PathBuf,
    total_size: u64,
    progress_emitter: UnboundedSender<FileActionProgress>,
) -> Result<()> {
    let stream = TcpStream::connect("127.0.0.1:8000").await?;
    let mut stream = BufReader::new(stream);
    let mut progress = FileActionProgress {
        operation: FileAction::Upload,
        total: total_size,
        ..Default::default()
    };
    for file in files
        .iter()
        .filter(|file| file.last_update.operation != FileOperation::Remove)
        .collect::<Vec<&RSFile>>()
    {
        send_file(
            &mut stream,
            file,
            upload_token,
            &root_folder,
            &mut progress,
            &progress_emitter,
        )
        .await?;
    }
    send_commit_msg(&mut stream, upload_token).await
}

pub async fn download_files(root: PathBuf, files: &[RSFile], download_token: String) -> Result<()> {
    let stream = TcpStream::connect("127.0.0.1:8000").await?;
    let mut stream = BufReader::new(stream);
    let mut _bytes_received: u64 = 0;
    for file in files
        .iter()
        .filter(|file| file.last_update.operation != FileOperation::Remove)
        .collect::<Vec<&RSFile>>()
    {
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
    delete_removed_files(&root, files).await?;
    Ok(())
}

async fn send_file<'a>(
    stream: &mut BufReader<TcpStream>,
    file: &RSFile,
    upload_token: &String,
    root_folder: &Path,
    file_action_progress: &'a mut FileActionProgress,
    progress_emitter: &'a UnboundedSender<FileActionProgress>,
) -> Result<()> {
    println!("Uploading {} file", file.path);
    file_action_progress.current_file_name = file.path.to_owned();

    let mut retry_count: u8 = 0;
    loop {
        let mut file_upload_message =
            FileUploadMessageFactory::new(upload_token, file, root_folder.to_path_buf());
        while file_upload_message.has_data_to_fetch() {
            let packet = file_upload_message.get_tcp_payload()?;
            send_message(stream.borrow_mut(), &packet).await?;

            file_action_progress.progress += file_upload_message.last_chunk_size as u64;
            let _ = progress_emitter.send(file_action_progress.clone());
            sleep(Duration::from_secs(1)).await;

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

async fn delete_removed_files(root: &Path, files: &[RSFile]) -> Result<()> {
    for file in files
        .iter()
        .filter(|f| f.last_update.operation == FileOperation::Remove)
    {
        let path = root.join(&file.path);
        tokio::fs::remove_file(&path).await?;
    }
    delete_empty_folders(root).await
}

#[async_recursion]
async fn delete_empty_folders(path: &Path) -> Result<()> {
    if let Ok(mut entries) = tokio::fs::read_dir(path).await {
        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                // Recursively delete empty subfolders
                delete_empty_folders(&entry_path).await?;

                // Delete current folder if it's empty
                if tokio::fs::read_dir(&entry_path)
                    .await?
                    .next_entry()
                    .await?
                    .is_none()
                {
                    tokio::fs::remove_dir(&entry_path).await?;
                }
            }
        }
    }
    Ok(())
}
