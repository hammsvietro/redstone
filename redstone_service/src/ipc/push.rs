use std::path::Path;

use interprocess::local_socket::LocalSocketStream;
use redstone_common::{
    model::{
        api::{
            Endpoints, FileUploadRequest, PushRequest as ApiPushRequest, Update, UploadResponse,
        },
        backup::{get_index_file_for_path, IndexFile},
        fs_tree::FSTree,
        ipc::{push::PushRequest as IpcPushRequest, IpcMessage, IpcMessageResponse},
        DomainError, RedstoneError, Result,
    },
    web::api::{handle_response, RedstoneClient},
};
use reqwest::Method;
pub async fn handle_push_msg(
    _connection: &mut LocalSocketStream,
    push_request: &mut IpcPushRequest,
) -> Result<IpcMessage> {
    let index_file = get_index_file(&push_request.path).await?;
    let client = RedstoneClient::new();

    let latest_update_response = client
        .send::<()>(
            Method::GET,
            Endpoints::FetchUpdate(index_file.backup.id.to_owned()).get_url(),
            &None,
        )
        .await?;

    let latest_update: Update = handle_response(latest_update_response).await?;
    // update_index_file()?;

    if latest_update.hash != index_file.current_update.hash {
        return wrap(IpcMessageResponse {
            message: None,
            keep_connection: false,
            error: Some(RedstoneError::DomainError(DomainError::NotInLatestUpdate)),
        });
    }

    let fs_tree = FSTree::build(push_request.path.clone(), None)?;
    let diff = fs_tree.diff(&index_file.last_fs_tree)?;
    if !diff.has_changes() {
        return wrap(IpcMessageResponse {
            message: None,
            keep_connection: false,
            error: Some(RedstoneError::DomainError(DomainError::NoChanges)),
        });
    }

    let request = ApiPushRequest {
        backup_id: index_file.backup.id.to_owned(),
        files: FileUploadRequest::from_diff(&diff),
    };
    let res = client
        .send(Method::POST, Endpoints::Push.get_url(), &Some(request))
        .await?;

    let _push_response: UploadResponse = handle_response(res).await?;

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
