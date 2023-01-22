use interprocess::local_socket::LocalSocketStream;

use redstone_common::model::{Result, ipc::{clone::CloneRequest, IpcMessage, IpcMessageResponse}};

pub async fn handle_clone_msg(_connection: &mut LocalSocketStream, clone_request: &mut CloneRequest) -> Result<IpcMessage> {
    println!("{:?}", clone_request);
    Ok(IpcMessage::Response(IpcMessageResponse { message: None, keep_connection: false, error: None }))
}
