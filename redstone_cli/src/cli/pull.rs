use std::env::current_dir;

use redstone_common::model::{
    backup::get_index_file_for_path,
    ipc::{
        pull::PullRequest, ConfirmationRequest, IpcMessage, IpcMessageRequest,
        IpcMessageRequestType, IpcMessageResponse,
    },
    DomainError, RedstoneError, Result,
};

use crate::{
    ipc::socket::{send_and_receive, stablish_connection},
    utils::handle_confirmation_request,
};

pub fn run_pull_cmd() -> Result<()> {
    let path = current_dir()?;
    let index_file_path = get_index_file_for_path(&path);
    if !index_file_path.exists() {
        let path = path.to_str().unwrap().into();
        return Err(RedstoneError::DomainError(DomainError::BackupDoesntExist(
            path,
        )));
    }
    let request = IpcMessage::Request(IpcMessageRequest {
        message: IpcMessageRequestType::PullRequest(PullRequest { path }),
    });
    let mut connection = stablish_connection()?;
    let received_message = send_and_receive(&mut connection, request)?;
    if received_message.has_errors() {
        let error = IpcMessageResponse::from(received_message).error.unwrap();
        return Err(error);
    }

    let confirmation_request: ConfirmationRequest = ConfirmationRequest::from(received_message);
    let confirmation_response =
        handle_confirmation_request(&mut connection, &confirmation_request)?;
    let received_message = send_and_receive(&mut connection, confirmation_response)?;
    if received_message.has_errors() {
        let error = IpcMessageResponse::from(received_message).error.unwrap();
        return Err(error);
    }

    Ok(())
}
