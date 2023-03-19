use std::borrow::BorrowMut;

use interprocess::local_socket::LocalSocketStream;
use redstone_common::{
    model::{
        api::{Endpoints, FileUploadRequest, PushRequest as ApiPushRequest, UploadResponse},
        backup::{get_index_file_for_path, IndexFile},
        fs_tree::FSTree,
        ipc::{
            push::PushRequest as IpcPushRequest, ConfirmationRequest, FileActionProgress,
            IpcMessage, IpcMessageResponse,
        },
        DomainError, RedstoneError, Result,
    },
    web::api::{handle_response, RedstoneClient},
};
use reqwest::Method;
use tokio::sync::mpsc;

use crate::{
    api::update::check_latest_update, backup::file_transfer::send_files, ipc::send_progress,
};

use super::prompt_action_confirmation;
pub async fn handle_push_msg(
    connection: &mut LocalSocketStream,
    push_request: &mut IpcPushRequest,
) -> Result<IpcMessage> {
    let index_file_path = get_index_file_for_path(&push_request.path);
    let mut index_file = IndexFile::from_file(&index_file_path)?;

    let latest_update = check_latest_update(index_file.backup.id.to_owned()).await?;
    index_file.latest_update = latest_update.clone();
    index_file.save(&index_file_path)?;

    if latest_update.hash != index_file.current_update.hash {
        return wrap(IpcMessageResponse {
            message: None,
            keep_connection: false,
            error: Some(RedstoneError::DomainError(DomainError::NotInLatestUpdate)),
        });
    }

    let fs_tree = FSTree::build(push_request.path.clone(), None)?;
    let diff = fs_tree.diff(&index_file.last_fs_tree)?;
    let total_size = diff.total_size();
    if !diff.has_changes() {
        return wrap(IpcMessageResponse {
            message: None,
            keep_connection: false,
            error: Some(RedstoneError::DomainError(DomainError::NoChanges)),
        });
    }
    let confirmation_request = ConfirmationRequest {
        message: format!(
            "The following changes will be made:\n{}",
            diff.get_changes_message()
        ),
    };
    let confirmation_result =
        prompt_action_confirmation(connection.borrow_mut(), confirmation_request).await?;
    if !confirmation_result.has_accepted {
        return wrap(IpcMessageResponse {
            keep_connection: false,
            error: None,
            message: None,
        });
    }

    let request = ApiPushRequest {
        backup_id: index_file.backup.id.to_owned(),
        files: FileUploadRequest::from_diff(&diff),
    };
    let client = RedstoneClient::new();
    let res = client
        .send(Method::POST, Endpoints::Push.get_url(), &Some(request))
        .await?;

    let push_response: UploadResponse = handle_response(res).await?;
    let (tx, mut rx) = mpsc::unbounded_channel::<FileActionProgress>();
    let (_, send_files_result) = tokio::join!(
        send_progress(connection.borrow_mut(), &mut rx),
        send_files(
            &push_response.files,
            &push_response.upload_token,
            push_request.path.clone(),
            total_size,
            tx,
        )
    );

    send_files_result?;

    let latest_update = check_latest_update(index_file.backup.id.to_owned()).await?;
    let index_file = IndexFile::new(
        push_response.backup.clone(),
        latest_update.clone(),
        latest_update,
        index_file.config,
        fs_tree.clone(),
    );
    index_file.save(&index_file_path)?;

    wrap(IpcMessageResponse {
        message: None,
        keep_connection: false,
        error: None,
    })
}

fn wrap(response: IpcMessageResponse) -> Result<IpcMessage> {
    Ok(IpcMessage::from(response))
}
