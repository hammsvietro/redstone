use interprocess::local_socket::LocalSocketStream;
use redstone_common::model::{
    ipc::{push::PushRequest, IpcMessage, IpcMessageResponse},
    Result,
};

pub async fn handle_push_msg(
    connection: &mut LocalSocketStream,
    clone_request: &mut PushRequest,
) -> Result<IpcMessage> {
    Ok(IpcMessage::Response(IpcMessageResponse {
        message: None,
        keep_connection: false,
        error: None,
    }))
}
