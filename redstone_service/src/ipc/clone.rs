use interprocess::local_socket::LocalSocketStream;

use redstone_common::{
    model::{
        api::{CloneRequest as ApiCloneRequest, Endpoints},
        ipc::{clone::CloneRequest, IpcMessage, IpcMessageResponse},
        Result,
    },
    web::api::{jar::get_jar, RedstoneClient},
};
use reqwest::Method;

pub async fn handle_clone_msg(
    _connection: &mut LocalSocketStream,
    clone_request: &mut CloneRequest,
) -> Result<IpcMessage> {
    println!("{:?}", clone_request);
    let client = RedstoneClient::new(get_jar()?);
    let request = &Some(ApiCloneRequest::new(clone_request.backup_name.clone()));
    let response = client
        .send(Method::POST, Endpoints::Clone.get_url(), request)
        .await?;
    Ok(IpcMessageResponse {
        message: None,
        keep_connection: false,
        error: None,
    }
    .into())
}
