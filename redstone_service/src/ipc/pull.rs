use std::borrow::BorrowMut;

use interprocess::local_socket::LocalSocketStream;
use redstone_common::{
    model::{
        api::{DownloadResponse, Endpoints, PullRequest as ApiPullRequest},
        backup::{get_index_file_for_path, IndexFile},
        ipc::{
            pull::PullRequest, ConfirmationRequest, FileActionProgress, IpcMessage,
            IpcMessageResponse,
        },
        DomainError, RedstoneError, Result,
    },
    util::bytes_to_human_readable,
    web::api::{handle_response, RedstoneClient},
};
use reqwest::Method;
use tokio::sync::mpsc;

use crate::{
    api::update::check_latest_update, backup::file_transfer::download_files, ipc::send_progress,
};

use super::{build_fs_tree_with_progress, prompt_action_confirmation};

pub async fn handle_pull_msg(
    connection: &mut LocalSocketStream,
    pull_request: &mut PullRequest,
) -> Result<IpcMessage> {
    let index_file_path = get_index_file_for_path(&pull_request.path);
    let mut index_file = IndexFile::from_file(&index_file_path)?;

    let latest_update = check_latest_update(index_file.backup.id.to_owned()).await?;
    index_file.latest_update = latest_update.clone();
    index_file.save(&index_file_path)?;
    if latest_update.hash == index_file.current_update.hash {
        return wrap(IpcMessageResponse {
            message: None,
            keep_connection: false,
            error: Some(RedstoneError::DomainError(
                DomainError::AlreadyInLatestUpdate,
            )),
        });
    }

    let client = RedstoneClient::new();
    let api_request = ApiPullRequest {
        backup_id: index_file.backup.id.to_owned(),
        update_id: index_file.current_update.id.to_owned(),
    };

    let response = client
        .send(Method::POST, Endpoints::Pull.get_url(), &Some(api_request))
        .await?;

    let download_response: DownloadResponse = handle_response(response).await?;

    let confirmation_request = ConfirmationRequest {
        message: get_confirmation_request_message(download_response.total_bytes),
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

    let (_, _) = tokio::join!(
        send_progress(connection.borrow_mut(), &mut rx),
        download_files(
            pull_request.path.clone(),
            &download_response.files,
            download_response.download_token.to_owned(),
            download_response.total_bytes as u64,
            tx
        )
    );

    index_file.current_update = download_response.update.clone();
    index_file.last_fs_tree =
        build_fs_tree_with_progress(connection, pull_request.path.clone()).await?;

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

fn get_confirmation_request_message(total_bytes: usize) -> String {
    let readable_bytes = bytes_to_human_readable(total_bytes);
    let message = format!("\nBy continuing, you will download {readable_bytes} of data");
    message
}
