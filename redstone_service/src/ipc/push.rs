use std::borrow::BorrowMut;

use interprocess::local_socket::LocalSocketStream;
use redstone_common::{
    model::{
        api::{Endpoints, FileUploadRequest, PushRequest as ApiPushRequest, UploadResponse},
        backup::{get_index_file_for_path, IndexFile},
        fs_tree::FSTree,
        ipc::{
            push::PushRequest as IpcPushRequest, ConfirmationRequest, IpcMessage,
            IpcMessageResponse,
        },
        DomainError, RedstoneError, Result,
    },
    web::api::{handle_response, RedstoneClient},
};
use reqwest::Method;
use tokio::sync::mpsc::{self, UnboundedReceiver};

use crate::{api::update::check_latest_update, backup::file_transfer::send_files};

use super::socket_loop::prompt_action_confirmation;
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
    println!("DIFF:");
    println!("{:?}", diff);
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
    let (tx, mut rx) = mpsc::unbounded_channel::<u64>();
    let (_, send_files_result) = tokio::join!(
        send_progress(&mut rx, total_size),
        send_files(
            &push_response.files,
            &push_response.upload_token,
            push_request.path.clone(),
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

async fn send_progress(_progress_receiver: &mut UnboundedReceiver<u64>, _total_size: u64) {
    // while let Some(sent) = progress_receiver.recv().await {
    //     println!("UPLOAD PROGRESS!\n{} sent out of {}", sent, total_size);
    //  // send to cli
    // }
}
