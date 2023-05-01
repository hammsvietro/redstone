use std::env::current_dir;

use redstone_common::{
    config::assert_configuration_and_authentication,
    ipc::send_and_receive,
    model::{
        backup::get_index_file_for_path,
        ipc::{
            clone::CloneRequest, ConfirmationRequest, IpcMessage, IpcMessageRequest,
            IpcMessageRequestType, IpcMessageResponse,
        },
        DomainError, RedstoneError, Result,
    },
};

use crate::{ipc::socket::stablish_connection, utils::handle_confirmation_request};

use super::{models::CloneArgs, progress_bar::handle_progress_bar};

pub fn run_clone_cmd(clone_args: CloneArgs) -> Result<()> {
    assert_configuration_and_authentication()?;
    let path = current_dir()?;
    let backup_name = clone_args.backup_name;
    let index_file_path = get_index_file_for_path(&path);
    if index_file_path.exists() {
        let path = path.to_str().unwrap().into();
        return Err(RedstoneError::DomainError(
            DomainError::BackupAlreadyExists(path),
        ));
    }

    let request = IpcMessage::Request(IpcMessageRequest {
        message: IpcMessageRequestType::CloneRequest(CloneRequest { path, backup_name }),
    });

    let mut connection = stablish_connection()?;
    let response = send_and_receive(&mut connection, &request)?;
    if response.has_errors() {
        let error = IpcMessageResponse::from(response).error.unwrap();
        return Err(error);
    }
    let confirmation_request: ConfirmationRequest = ConfirmationRequest::from(response);
    let confirmation_response =
        handle_confirmation_request(&mut connection, &confirmation_request)?;

    let mut received_message = send_and_receive(&mut connection, &confirmation_response)?;
    received_message = handle_progress_bar(&mut connection, received_message)?;
    if received_message.has_errors() {
        let error = IpcMessageResponse::from(received_message).error.unwrap();
        return Err(error);
    }

    Ok(())
}
