use std::env::current_dir;

use redstone_common::model::{Result, backup::get_index_file_for_path, RedstoneError, DomainError, ipc::{IpcMessageRequestType, IpcMessage, IpcMessageRequest, clone::CloneRequest}};

use crate::ipc::socket::{stablish_connection, send_and_receive};

use super::models::CloneArgs;

pub fn run_clone_cmd(clone_args: CloneArgs) -> Result<()> {
    let path = current_dir()?;
    let backup_name = clone_args.backup_name;
    let index_file_path = get_index_file_for_path(&path);
    if index_file_path.exists() {
        let path = path.to_str().unwrap().into();
        return Err(RedstoneError::DomainError(DomainError::DirectoryAlreadyBeingTracked(path)))
    }

    let request = IpcMessage::Request(IpcMessageRequest {
        message: IpcMessageRequestType::CloneRequest(CloneRequest { path, backup_name })
    });

    let mut connection = stablish_connection()?;
    let response = send_and_receive(&mut connection, request)?;
    println!("{:?}", response);
    
    Ok(())
}
