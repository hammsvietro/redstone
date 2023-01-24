use std::{borrow::BorrowMut, path::PathBuf};

use interprocess::local_socket::LocalSocketStream;

use redstone_common::{
    model::{
        api::{CloneRequest as ApiCloneRequest, CloneResponse, Endpoints, File},
        fs_tree::{FSTree, RSFile},
        ipc::{clone::CloneRequest, ConfirmationRequest, IpcMessage, IpcMessageResponse},
        Result,
    },
    util::bytes_to_human_readable,
    web::api::{jar::get_jar, RedstoneClient},
};
use reqwest::Method;

use crate::ipc::socket_loop::prompt_action_confirmation;

pub async fn handle_clone_msg(
    connection: &mut LocalSocketStream,
    clone_request: &mut CloneRequest,
) -> Result<IpcMessage> {
    println!("{:?}", clone_request);
    let client = RedstoneClient::new(get_jar()?);
    let request = &Some(ApiCloneRequest::new(clone_request.backup_name.clone()));
    let response = client
        .send(Method::POST, Endpoints::Clone.get_url(), request)
        .await?;

    let clone_response: CloneResponse = response.json().await?;

    let conflicting_files =
        get_conflicting_files(&clone_request.path, &clone_response.files_to_download)?;
    let confirmation_request = ConfirmationRequest {
        message: get_confirmation_request_message(conflicting_files, clone_response.total_bytes),
    };
    let confirmation_result =
        match prompt_action_confirmation(connection.borrow_mut(), confirmation_request).await {
            Ok(confirmation_response) => confirmation_response,
            Err(err) => return Err(err),
        };
    if !confirmation_result.has_accepted {
        return Ok(IpcMessageResponse {
            message: None,
            keep_connection: false,
            error: None,
        }
        .into());
    }

    Ok(IpcMessageResponse {
        message: None,
        keep_connection: false,
        error: None,
    }
    .into())
}

fn get_conflicting_files(path: &PathBuf, api_files: &Vec<File>) -> Result<Vec<RSFile>> {
    let fs_tree = FSTree::build(path.clone(), None)?;
    let api_file_paths: Vec<String> = api_files.iter().map(|file| file.path.clone()).collect();
    Ok(fs_tree.get_conflicting_files(api_file_paths))
}

fn get_confirmation_request_message(conflicting_files: Vec<RSFile>, total_bytes: usize) -> String {
    let readable_bytes = bytes_to_human_readable(total_bytes);
    let mut message = format!(
        "\nBy continuing, you will download {} of data",
        readable_bytes
    );
    conflicting_files
        .iter()
        .enumerate()
        .for_each(|(idx, file)| {
            if idx == 0 {
                message += "The following files will be overwritten:\n";
            }
            message += &format!("{}\n", file.path);
        });
    message += "\nDo you wish to continue?";
    message
}
