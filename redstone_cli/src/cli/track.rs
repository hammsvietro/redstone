use std::{env::current_dir, path::PathBuf, str::FromStr};

use redstone_common::model::{
    ipc::{
        ConfirmationRequest, IpcMessage, IpcMessageRequest, IpcMessageRequestType,
        IpcMessageResponse,
    },
    track::TrackRequest,
    RedstoneError, Result,
};

use crate::{
    ipc::socket::{send_and_receive, stablish_connection},
    utils::handle_confirmation_request,
};

use super::models::TrackArgs;

pub fn run_track_cmd(track_args: TrackArgs) -> Result<()> {
    let path_buf = get_target_path(track_args.path);
    let track_request = TrackRequest {
        base_path: path_buf,
        name: track_args.name,
        detatched: track_args.detached,
        sync_every: track_args.sync_every,
        watch: track_args.watch,
    };
    let request = IpcMessage::Request(IpcMessageRequest {
        message: IpcMessageRequestType::TrackRequest(track_request),
    });

    let mut conn = stablish_connection()?;
    let received_message = send_and_receive(&mut conn, request)?;
    let confirmation_request: ConfirmationRequest = ConfirmationRequest::from(received_message);
    let confirmation_response = handle_confirmation_request(&confirmation_request)?;
    if !confirmation_response.keep_connection {
        if let Some(err) = confirmation_response.error {
            return Err(RedstoneError::from(err));
        }
        return Ok(());
    }
    let received_message = send_and_receive(&mut conn, IpcMessage::from(confirmation_response));
    if let Err(err) = received_message {
        return Err(err);
    }
    let received_message = received_message.unwrap();
    if received_message.is_response() {
        let response = IpcMessageResponse::from(received_message);
        if let Some(err) = response.error {
            return Err(RedstoneError::from(err));
        }
    }
    Ok(())
}

fn get_target_path(path_string: Option<String>) -> PathBuf {
    let mut current_directory = current_dir().unwrap();
    if path_string.is_none() {
        return current_directory;
    }
    let path_string = path_string.unwrap();
    if path_string.starts_with('~') || path_string.starts_with('/') {
        return PathBuf::from_str(&path_string).unwrap();
    }
    current_directory.push(path_string);
    current_directory
}
