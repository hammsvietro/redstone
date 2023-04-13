use interprocess::local_socket::LocalSocketStream;
use redstone_common::{constants::IPC_SOCKET_PATH, model::Result};

pub fn stablish_connection() -> Result<LocalSocketStream> {
    Ok(LocalSocketStream::connect(IPC_SOCKET_PATH)?)
}
