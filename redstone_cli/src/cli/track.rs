use std::{env::current_dir, path::PathBuf, str::FromStr};

use redstone_common::{
    config::assert_configuration_and_authentication,
    ipc::send_and_receive,
    model::{
        ipc::track::TrackRequest,
        ipc::{
            ConfirmationRequest, IpcMessage, IpcMessageRequest, IpcMessageRequestType,
            IpcMessageResponse,
        },
        Result,
    },
};

use crate::{ipc::socket::stablish_connection, utils::handle_confirmation_request};

use super::{models::TrackArgs, progress_bar::handle_progress_bar};

pub fn run_track_cmd(track_args: TrackArgs) -> Result<()> {
    assert_configuration_and_authentication()?;
    let path_buf = get_target_path(track_args.path);
    let track_request = TrackRequest {
        base_path: path_buf,
        name: track_args.backup_name,
        detatched: track_args.detached,
        sync_every: track_args.sync_every,
        watch: track_args.watch,
    };
    let request = IpcMessage::Request(IpcMessageRequest {
        message: IpcMessageRequestType::TrackRequest(track_request),
    });

    let mut conn = stablish_connection()?;
    let mut received_message = send_and_receive(&mut conn, &request)?;
    if received_message.has_errors() {
        let error = IpcMessageResponse::from(received_message).error.unwrap();
        return Err(error);
    }
    received_message = handle_progress_bar(&mut conn, received_message)?;
    let confirmation_request: ConfirmationRequest = ConfirmationRequest::from(received_message);
    let confirmation_response = handle_confirmation_request(&mut conn, &confirmation_request)?;
    let mut received_message = send_and_receive(&mut conn, &confirmation_response)?;

    received_message = handle_progress_bar(&mut conn, received_message)?;

    if received_message.is_response() {
        let response = IpcMessageResponse::from(received_message);
        if let Some(err) = response.error {
            return Err(err);
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
