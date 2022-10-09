use interprocess::local_socket::LocalSocketStream;
use redstone_common::{model::{
    backup::IndexFile,
    fs_tree::{FSTree, FSTreeItem},
    ipc::{ConfirmationRequest, IpcMessage, IpcMessageResponse, IpcMessageResponseType},
    track::{TrackMessageResponse, TrackRequest},
}, api::{get_http_client, jar::get_jar, get_api_base_url}};
use std::{borrow::BorrowMut, io::Write, path::PathBuf};

use super::socket_loop::prompt_action_confirmation;

pub async fn handle_track_msg(
    connection: &mut LocalSocketStream,
    track_request: &mut TrackRequest,
) -> Result<IpcMessage, Box<dyn std::fmt::Debug + Send>> {
    let message = TrackMessageResponse {
        data: String::from("=)"),
    };
    let fs_tree = FSTree::new(track_request.base_path.clone(), None);
    let index_file_path = fs_tree.get_index_file_for_root();
    if index_file_path.exists() {
        return wrap(IpcMessageResponse {
            keep_connection: false,
            error: Some(String::from("Directory specified is already being tracked")),
            message: Some(IpcMessageResponseType::TrackMessageResponse(message)),
        });
    }
    //
    // TODO: ping server, check for resource availability and request backup id
    let confirmation_request = ConfirmationRequest {
        message: get_confirmation_request_message(&fs_tree),
    };
    let confirmation_result =
        match prompt_action_confirmation(connection.borrow_mut(), confirmation_request).await {
            Ok(confirmation_response) => confirmation_response,
            Err(err) => return Err(Box::new(err)),
        };
    if !confirmation_result.has_accepted {
        return wrap(IpcMessageResponse {
            keep_connection: false,
            error: None,
            message: Some(IpcMessageResponseType::TrackMessageResponse(message)),
        });
    }
    create_files(&index_file_path, track_request.borrow_mut(), &fs_tree).unwrap();

    // let cookie_jar = get_jar().unwrap();
    // get_http_client(cookie_jar);
    // let base_url = get_api_base_url();

    wrap(IpcMessageResponse {
        keep_connection: false,
        error: None,
        message: Some(IpcMessageResponseType::TrackMessageResponse(message)),
    })
}

fn wrap(response: IpcMessageResponse) -> Result<IpcMessage, Box<dyn std::fmt::Debug + Send>> {
    Ok(response.into())
}

fn get_confirmation_request_message(fs_tree: &FSTree) -> String {
    let mut message = String::from("By continuing, you will recursively backup the following:\n");
    for item in fs_tree.get_first_depth() {
        message += match item {
            FSTreeItem::File(file) => file.path.as_str(),
            FSTreeItem::Folder(folder) => folder.path.as_str(),
        };
        message += "\n"
    }
    message += "\nDo you wish to continue?";
    message
}

fn create_files(
    index_file_path: &PathBuf,
    track_request: &mut TrackRequest,
    fs_tree: &FSTree,
) -> std::io::Result<()> {
    let parent_folders = index_file_path.parent();
    if let Some(folder_path) = parent_folders {
        std::fs::create_dir_all(folder_path).unwrap();
    }
    let mut index_file = std::fs::File::create(index_file_path).unwrap();
    let index_file_content =
        bincode::serialize(&IndexFile::new(track_request.clone(), fs_tree)).unwrap();
    index_file.write_all(&index_file_content)
}
