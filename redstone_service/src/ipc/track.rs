use interprocess::local_socket::LocalSocketStream;
use redstone_common::{
    model::{
        api::{DeclareBackupRequest, Endpoints, FileUploadRequest, UploadResponse},
        backup::{get_index_file_for_path, BackupConfig, IndexFile},
        fs_tree::{FSTree, FSTreeDiff},
        ipc::{track::TrackRequest, FileActionProgress},
        ipc::{ConfirmationRequest, IpcMessage, IpcMessageResponse},
        DomainError, RedstoneError, Result,
    },
    web::api::{handle_response, RedstoneClient},
};
use reqwest::Method;
use std::{borrow::BorrowMut, io::Write, path::PathBuf};
use tokio::sync::mpsc;

use crate::{backup::file_transfer::send_files, ipc::send_progress};

use super::{build_fs_tree_with_progress, prompt_action_confirmation};

pub async fn handle_track_msg(
    connection: &mut LocalSocketStream,
    track_request: &mut TrackRequest,
) -> Result<IpcMessage> {
    let base_path = &track_request.base_path;
    let index_file_path = get_index_file_for_path(base_path);
    if index_file_path.exists() {
        let path = base_path.to_str().unwrap().into();
        return wrap(IpcMessageResponse {
            keep_connection: false,
            error: Some(RedstoneError::DomainError(
                DomainError::BackupAlreadyExists(path),
            )),
            message: None,
        });
    }
    let fs_tree = build_fs_tree_with_progress(connection, track_request.base_path.clone()).await?;
    let confirmation_request = get_confirmation_message(&fs_tree);
    let confirmation_result =
        prompt_action_confirmation(connection.borrow_mut(), confirmation_request).await?;
    if !confirmation_result.has_accepted {
        return wrap(IpcMessageResponse {
            keep_connection: false,
            error: None,
            message: None,
        });
    }

    let total_size = fs_tree.total_size();
    let root_folder = fs_tree.root.clone();
    let files = fs_tree
        .files
        .iter()
        .map(|file| FileUploadRequest::from(file.clone()))
        .collect();

    let declare_request =
        DeclareBackupRequest::new(track_request.name.as_str(), fs_tree.root.clone(), files);

    let declare_response = declare(&declare_request).await?;
    let (tx, mut rx) = mpsc::unbounded_channel::<FileActionProgress>();
    let (_, send_files_result) = tokio::join!(
        send_progress(connection.borrow_mut(), &mut rx),
        send_files(
            &declare_response.files,
            &declare_response.upload_token,
            root_folder,
            total_size,
            tx
        )
    );
    send_files_result?;

    create_files(
        &index_file_path,
        declare_response,
        track_request.borrow_mut(),
        fs_tree,
    )?;
    wrap(IpcMessageResponse {
        keep_connection: false,
        error: None,
        message: None,
    })
}

fn wrap(response: IpcMessageResponse) -> Result<IpcMessage> {
    Ok(response.into())
}

async fn declare<'a>(request: &'a DeclareBackupRequest<'a>) -> Result<UploadResponse> {
    let client = RedstoneClient::new();

    let response = client
        .send(Method::POST, Endpoints::Declare.get_url()?, &Some(request))
        .await?;

    handle_response(response).await
}
fn get_confirmation_message(fs_tree: &FSTree) -> ConfirmationRequest {
    let diff = FSTreeDiff {
        new_files: fs_tree.files.clone(),
        ..Default::default()
    };

    let message = format!(
        "By continuing, you will recursively backup the following:\n{}",
        diff.get_changes_message()
    );

    ConfirmationRequest { message }
}

fn create_files(
    index_file_path: &PathBuf,
    declare_response: UploadResponse,
    track_request: &mut TrackRequest,
    fs_tree: FSTree,
) -> Result<IndexFile> {
    let parent_folders = index_file_path.parent();
    if let Some(folder_path) = parent_folders {
        std::fs::create_dir_all(folder_path)?;
    }
    let mut index_file = std::fs::File::create(index_file_path)?;
    let config = BackupConfig::new(track_request.sync_every.clone(), track_request.watch);
    let index_file_content = IndexFile::new(
        declare_response.backup,
        declare_response.update.clone(),
        declare_response.update,
        config,
        fs_tree,
    );
    index_file.write_all(&bincode::serialize(&index_file_content)?)?;
    Ok(index_file_content)
}
