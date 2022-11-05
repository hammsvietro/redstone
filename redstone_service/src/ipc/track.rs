use interprocess::local_socket::LocalSocketStream;
use redstone_common::{
    api::{get_api_base_url, get_http_client, jar::get_jar},
    model::{
        api::{DeclareBackupRequest, DeclareBackupResponse},
        backup::IndexFile,
        fs_tree::FSTree,
        ipc::{ConfirmationRequest, IpcMessage, IpcMessageResponse, IpcMessageResponseType},
        track::{TrackMessageResponse, TrackRequest},
        Result,
    },
};
use std::{borrow::BorrowMut, collections::HashSet, io::Write, path::PathBuf};
use tokio::{
    io::AsyncWriteExt,
    sync::mpsc::{self, UnboundedReceiver},
};

use crate::backup::file_transfer::send_files;

use super::socket_loop::prompt_action_confirmation;

pub async fn handle_track_msg(
    connection: &mut LocalSocketStream,
    track_request: &mut TrackRequest,
) -> Result<IpcMessage> {
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

    let confirmation_request = ConfirmationRequest {
        message: get_confirmation_request_message(&fs_tree),
    };
    let confirmation_result =
        match prompt_action_confirmation(connection.borrow_mut(), confirmation_request).await {
            Ok(confirmation_response) => confirmation_response,
            Err(err) => return Err(err),
        };
    if !confirmation_result.has_accepted {
        return wrap(IpcMessageResponse {
            keep_connection: false,
            error: None,
            message: Some(IpcMessageResponseType::TrackMessageResponse(message)),
        });
    }

    let total_size = fs_tree.total_size();
    let request = DeclareBackupRequest::new(String::from("test"), fs_tree.root, fs_tree.files);

    let cookie_jar = get_jar().unwrap();
    let base_url = get_api_base_url();
    let declare_response: DeclareBackupResponse = get_http_client(cookie_jar)
        .post(base_url.join("/api/upload/declare").unwrap())
        .json(&request)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    println!("{:?}", declare_response);
    // create_files(&index_file_path, res, track_request.borrow_mut()).unwrap();
    let (tx, mut rx) = mpsc::unbounded_channel::<u64>();
    let (_, _) = tokio::join!(
        send_files(
            declare_response.backup.files,
            declare_response.update_token,
            tx
        ),
        send_progress(&mut rx, total_size)
    );
    wrap(IpcMessageResponse {
        keep_connection: false,
        error: None,
        message: Some(IpcMessageResponseType::TrackMessageResponse(message)),
    })
}

fn wrap(response: IpcMessageResponse) -> Result<IpcMessage> {
    Ok(response.into())
}

async fn send_progress(progress_receiver: &mut UnboundedReceiver<u64>, total_size: u64) {
    while let Some(sent) = progress_receiver.recv().await {
        println!("UPLOAD PROGRESS!\n{} sent out of {}", sent, total_size);
        // send to cli
    }
}

fn get_confirmation_request_message(fs_tree: &FSTree) -> String {
    let mut message = String::from("By continuing, you will recursively backup the following:\n");
    let mut first_depth_file_structure: HashSet<String> = HashSet::new();
    for item in fs_tree.get_first_depth() {
        let file_path = item.path.as_str();
        let formatted_path = match file_path.split_once("/") {
            None => String::from(file_path),
            Some((before, _after)) => String::from(before) + "/",
        };
        first_depth_file_structure.insert(formatted_path);
    }
    for file_path in first_depth_file_structure {
        message += &file_path;
        message += "\n"
    }
    message += "\nDo you wish to continue?";
    message
}

fn create_files(
    index_file_path: &PathBuf,
    declare_response: DeclareBackupResponse,
    track_request: &mut TrackRequest,
) -> Result<IndexFile> {
    let parent_folders = index_file_path.parent();
    if let Some(folder_path) = parent_folders {
        std::fs::create_dir_all(folder_path).unwrap();
    }
    let mut index_file = std::fs::File::create(index_file_path).unwrap();
    let index_file_content = IndexFile::new(declare_response, track_request);
    index_file.write_all(&bincode::serialize(&index_file_content).unwrap())?;
    Ok(index_file_content)
}
