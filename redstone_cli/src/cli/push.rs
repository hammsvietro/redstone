use std::env::current_dir;

use redstone_common::model::{
    backup::get_index_file_for_path,
    ipc::{
        push::PushRequest, IpcMessage, IpcMessageRequest, IpcMessageRequestType, IpcMessageResponse,
    },
    DomainError, RedstoneError, Result,
};

use crate::ipc::socket::{send_and_receive, stablish_connection};

pub fn run_push_cmd() -> Result<()> {
    let path = current_dir()?;
    let index_file_path = get_index_file_for_path(&path);
    if !index_file_path.exists() {
        let path = path.to_str().unwrap().into();
        return Err(RedstoneError::DomainError(DomainError::BackupDoesntExist(
            path,
        )));
    }

    let request = IpcMessage::Request(IpcMessageRequest {
        message: IpcMessageRequestType::PushRequest(PushRequest { path }),
    });
    let mut connection = stablish_connection()?;
    let response = send_and_receive(&mut connection, request)?;
    if response.has_errors() {
        let error = IpcMessageResponse::from(response).error.unwrap();
        return Err(error);
    }
    Ok(())
}
