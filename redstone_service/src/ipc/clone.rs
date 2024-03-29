use std::{borrow::BorrowMut, path::Path};

use interprocess::local_socket::LocalSocketStream;

use redstone_common::{
    model::{
        api::{CloneRequest as ApiCloneRequest, DownloadResponse, Endpoints, File},
        backup::{get_index_file_for_path, BackupConfig, IndexFile},
        fs_tree::{FSTree, RSFile},
        ipc::{
            clone::CloneRequest, ConfirmationRequest, FileActionProgress, IpcMessage,
            IpcMessageResponse,
        },
        Result,
    },
    util::bytes_to_human_readable,
    web::api::{handle_response, RedstoneClient},
};
use reqwest::Method;
use tokio::{io::AsyncWriteExt, sync::mpsc};

use crate::{
    backup::file_transfer::download_files,
    ipc::{prompt_action_confirmation, send_progress},
};

use super::build_fs_tree_with_progress;

pub async fn handle_clone_msg(
    connection: &mut LocalSocketStream,
    clone_request: &mut CloneRequest,
) -> Result<IpcMessage> {
    let client = RedstoneClient::new();
    let request = &Some(ApiCloneRequest::new(clone_request.backup_name.clone()));
    let response = client
        .send(Method::POST, Endpoints::Clone.get_url()?, request)
        .await?;

    let clone_response: DownloadResponse = handle_response(response).await?;

    let conflicting_files = get_conflicting_files(&clone_request.path, &clone_response.files)?;
    let confirmation_request = ConfirmationRequest {
        message: get_confirmation_request_message(conflicting_files, clone_response.total_bytes),
    };
    let confirmation_result =
        prompt_action_confirmation(connection.borrow_mut(), confirmation_request).await?;
    if !confirmation_result.has_accepted {
        return Ok(IpcMessageResponse {
            message: None,
            keep_connection: false,
            error: None,
        }
        .into());
    }

    let (tx, mut rx) = mpsc::unbounded_channel::<FileActionProgress>();
    let (_, download_result) = tokio::join!(
        send_progress(connection.borrow_mut(), &mut rx),
        download_files(
            clone_request.path.clone(),
            &clone_response.files,
            clone_response.download_token.clone(),
            clone_response.total_bytes as u64,
            tx
        )
    );

    download_result?;

    let fs_tree = build_fs_tree_with_progress(connection, clone_request.path.clone()).await?;

    write_index_file(clone_request.borrow_mut(), &clone_response, fs_tree).await?;

    Ok(IpcMessageResponse {
        message: None,
        keep_connection: false,
        error: None,
    }
    .into())
}

fn get_conflicting_files(path: &Path, api_files: &[File]) -> Result<Vec<RSFile>> {
    let fs_tree = FSTree::build(path.to_path_buf(), None)?;
    let api_file_paths: Vec<String> = api_files.iter().map(|file| file.path.clone()).collect();
    Ok(fs_tree.get_conflicting_files(api_file_paths))
}

fn get_confirmation_request_message(conflicting_files: Vec<RSFile>, total_bytes: usize) -> String {
    let readable_bytes = bytes_to_human_readable(total_bytes);
    let mut message = format!("\nBy continuing, you will download {readable_bytes} of data");
    conflicting_files
        .iter()
        .enumerate()
        .for_each(|(idx, file)| {
            if idx == 0 {
                message += "The following files will be overwritten:\n";
            }
            message += &format!("{}\n", file.path);
        });
    message
}

async fn write_index_file(
    clone_request: &mut CloneRequest,
    clone_response: &DownloadResponse,
    fs_tree: FSTree,
) -> Result<()> {
    let backup_config = BackupConfig::new(None, false);
    let index_file = IndexFile::new(
        clone_response.backup.clone(),
        clone_response.update.clone(),
        clone_response.update.clone(),
        backup_config,
        fs_tree,
    );
    let index_file_path = get_index_file_for_path(&clone_request.path);
    if !index_file_path.exists() {
        if let Some(parent_folders) = index_file_path.parent() {
            tokio::fs::create_dir_all(&parent_folders).await?;
        }
    }
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&index_file_path)
        .await?;

    file.write_all(&bincode::serialize(&index_file)?).await?;
    Ok(())
}
