use std::{env::current_dir, path::PathBuf, str::FromStr};

use redstone_common::model::{
    ipc::{ConfirmationRequest, IpcMessage, IpcMessageRequest, IpcMessageRequestType},
    track::TrackRequest,
    Result,
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
        return Ok(());
    }
    let _received_message = send_and_receive(&mut conn, IpcMessage::from(confirmation_response));
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
