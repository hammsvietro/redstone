use std::path::Path;

use interprocess::local_socket::LocalSocketStream;
use redstone_common::model::{
    backup::{get_index_file_for_path, IndexFile},
    ipc::{push::PushRequest, IpcMessage, IpcMessageResponse},
    DomainError, RedstoneError, Result,
};
use tokio::io::AsyncReadExt;

pub async fn handle_push_msg(
    connection: &mut LocalSocketStream,
    clone_request: &mut PushRequest,
) -> Result<IpcMessage> {
    let index_file = get_index_file(&clone_request.path).await?;
    println!("{:?}", index_file);

    Ok(IpcMessage::Response(IpcMessageResponse {
        message: None,
        keep_connection: false,
        error: None,
    }))
}

async fn get_index_file(path: &Path) -> Result<IndexFile> {
    let index_path = get_index_file_for_path(path);
    IndexFile::from_file(&index_path)
}
