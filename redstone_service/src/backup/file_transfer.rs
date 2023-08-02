use std::{
    borrow::BorrowMut,
    path::{Path, PathBuf},
};

use async_recursion::async_recursion;
use redstone_common::{
    constants::TCP_FILE_CHUNK_SIZE,
    model::{
        api::{File as RSFile, FileOperation},
        ipc::{FileAction, FileActionProgress},
        socket::{SocketMessage, SocketMessageResponse, SocketMessageResponseStatus},
        RedstoneError, Result,
    },
    web::{
        api::get_websockets_base_url,
        socket::{
            receive_message, receive_raw_message, send_message, CheckFileMessageFactory,
            CommitMessageFactory, DownloadChunkMessageFactory, FileUploadMessageFactory,
            FinishDownloadMessageFactory,
        },
    },
};

use tokio::{io::AsyncWriteExt, sync::mpsc::UnboundedSender};
use websocket::{
    sync::{stream::NetworkStream, Client},
    ClientBuilder, OwnedMessage,
};

pub async fn send_files(
    files: &[RSFile],
    upload_token: &String,
    root_folder: PathBuf,
    total_size: u64,
    progress_emitter: UnboundedSender<FileActionProgress>,
) -> Result<()> {
    let url = get_websockets_base_url()?;
    let mut client = ClientBuilder::new(&url)?.connect(None)?;
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
            &mut client,
            file,
            upload_token,
            &root_folder,
            &mut progress,
            &progress_emitter,
        )
        .await?;
    }
    send_commit_msg(&mut client, upload_token).await
}

pub async fn download_files(
    root: PathBuf,
    files: &[RSFile],
    download_token: String,
    total_size: u64,
    progress_emitter: UnboundedSender<FileActionProgress>,
) -> Result<()> {
    let mut client = ClientBuilder::new(&get_websockets_base_url()?)?.connect(None)?;
    let mut progress = FileActionProgress {
        total: total_size,
        operation: FileAction::Download,
        progress: 0,
        current_file_name: String::new(),
    };
    for file in files
        .iter()
        .filter(|file| file.last_update.operation != FileOperation::Remove)
        .collect::<Vec<&RSFile>>()
    {
        progress.current_file_name = file.path.to_owned();
        download_file(
            &mut client,
            file,
            &root,
            download_token.clone(),
            &mut progress,
            &progress_emitter,
        )
        .await?;
        println!("downloaded {}", file.path);
    }
    let packet = FinishDownloadMessageFactory::new(download_token.to_string()).get_payload()?;
    send_message(&mut client, &packet)?;
    let response: SocketMessageResponse<Vec<u8>> = receive_message(&mut client)?;
    if response.status != SocketMessageResponseStatus::Ok {
        let error = format!(
            "Error commiting finalizing download.\nServer responded: {}",
            response.reason.unwrap()
        );
        return Err(RedstoneError::BaseError(error));
    }
    delete_removed_files(&root, files).await?;
    progress.progress = progress.total;
    let _ = progress_emitter.send(progress);
    Ok(())
}

async fn send_file<'a>(
    client: &mut Client<Box<dyn NetworkStream + std::marker::Send>>,
    file: &RSFile,
    upload_token: &String,
    root_folder: &Path,
    file_action_progress: &'a mut FileActionProgress,
    progress_emitter: &'a UnboundedSender<FileActionProgress>,
) -> Result<()> {
    println!("Uploading {} file", file.path);
    file_action_progress.current_file_name = file.path.to_owned();

    let mut retry_count: u8 = 0;
    // TODO: Send PrepareFileUploadMessage here
    loop {
        let mut file_upload_message =
            FileUploadMessageFactory::new(file, root_folder.to_path_buf());
        while file_upload_message.has_data_to_fetch() {
            let packet = file_upload_message.get_payload()?;
            send_message(client.borrow_mut(), &packet)?;

            file_action_progress.progress += file_upload_message.last_chunk_size as u64;
            let _ = progress_emitter.send(file_action_progress.clone());

            let response: SocketMessageResponse<()> = receive_message(client.borrow_mut())?;
            if response.status != SocketMessageResponseStatus::Ok {
                let error = format!(
                    "Error commiting backup transaction.\nServer responded: {}",
                    response.reason.unwrap()
                );
                return Err(RedstoneError::BaseError(error));
            }
        }
        let check_file_message =
            CheckFileMessageFactory::new(upload_token, &file.id).get_payload()?;
        send_message(client.borrow_mut(), &check_file_message)?;
        let response: SocketMessageResponse<()> = receive_message(client.borrow_mut())?;
        match response.status {
            SocketMessageResponseStatus::Error => {
                return Err(RedstoneError::BaseError(format!(
                    "Server returned: {:?}",
                    response.reason.unwrap()
                )))
            }
            SocketMessageResponseStatus::Ok
                if response.retry.is_some() && response.retry.unwrap() =>
            {
                retry_count += 1;
                if retry_count > 4 {
                    return Err(RedstoneError::BaseError(format!(
                        "Retry count exceeded when checking file {}",
                        file.path
                    )));
                }
            }
            _ => {
                println!("{} upload complete\n", file.path);
                break;
            }
        };
    }
    Ok(())
}

async fn send_commit_msg(
    client: &mut Client<Box<dyn NetworkStream + std::marker::Send>>,
    upload_token: &str,
) -> Result<()> {
    let commit_payload = CommitMessageFactory::new(upload_token.to_owned()).get_payload()?;
    println!("Sending commit msg!");
    send_message(client.borrow_mut(), &commit_payload)?;
    let response: SocketMessageResponse<()> = receive_message(client.borrow_mut())?;
    if response.status != SocketMessageResponseStatus::Ok {
        let error = format!(
            "Error commiting backup transaction.\nServer responded: {}",
            response.reason.unwrap()
        );
        return Err(RedstoneError::BaseError(error));
    }
    Ok(())
}

async fn download_file<'a>(
    client: &mut Client<Box<dyn NetworkStream + std::marker::Send>>,
    file: &RSFile,
    root: &Path,
    download_token: String,
    file_action_progress: &'a mut FileActionProgress,
    progress_emitter: &'a UnboundedSender<FileActionProgress>,
) -> Result<()> {
    file_action_progress.current_file_name = file.path.to_owned();
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
        let packet = factory.get_payload()?;
        send_message(client.borrow_mut(), &packet)?;
        let message = receive_raw_message(client.borrow_mut())?;
        let OwnedMessage::Binary(data) = message else {
            unreachable!()
        };
        let mut file = tokio::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&path)
            .await?;
        file.write_all(&data).await?;
        file_action_progress.progress += data.len() as u64;
        let _ = progress_emitter.send(file_action_progress.clone());
        if data.len() < TCP_FILE_CHUNK_SIZE {
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
