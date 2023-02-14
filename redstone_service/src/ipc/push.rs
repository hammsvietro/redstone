use std::path::Path;

use interprocess::local_socket::LocalSocketStream;
use redstone_common::{
    model::{
        api::Endpoints,
        backup::{get_index_file_for_path, IndexFile},
        fs_tree::FSTree,
        ipc::{push::PushRequest, IpcMessage, IpcMessageResponse},
        DomainError, RedstoneError, Result,
    },
    web::api::RedstoneClient,
};
use reqwest::Method;
pub async fn handle_push_msg(
    _connection: &mut LocalSocketStream,
    clone_request: &mut PushRequest,
) -> Result<IpcMessage> {
    let index_file = get_index_file(&clone_request.path).await?;
    let fs_tree = FSTree::build(clone_request.path.clone(), None)?;
    let diff = fs_tree.diff(&index_file.last_fs_tree)?;
    if !diff.has_changes() {
        return wrap(IpcMessageResponse {
            message: None,
            keep_connection: false,
            error: Some(RedstoneError::DomainError(DomainError::NoChanges)),
        });
    }

    let client = RedstoneClient::new();
    let res = client
        .send::<()>(Method::POST, Endpoints::Push.get_url(), &None)
        .await?;

    wrap(IpcMessageResponse {
        message: None,
        keep_connection: false,
        error: None,
    })
}

async fn get_index_file(path: &Path) -> Result<IndexFile> {
    let index_path = get_index_file_for_path(path);
    IndexFile::from_file(&index_path)
}

fn wrap(response: IpcMessageResponse) -> Result<IpcMessage> {
    Ok(IpcMessage::from(response))
}
