use interprocess::local_socket::LocalSocketStream;
use redstone_common::{
    constants::{IPC_BUFFER_SIZE, IPC_SOCKET_PATH},
    model::ipc::IpcMessage,
};
use std::{
    borrow::BorrowMut,
    io::{prelude::*, BufReader},
};

pub fn stablish_connection() -> std::io::Result<LocalSocketStream> {
    LocalSocketStream::connect(IPC_SOCKET_PATH)
}

pub fn send_and_receive(
    conn: &mut LocalSocketStream,
    ipc_message: IpcMessage,
) -> std::io::Result<IpcMessage> {
    let mut buffer = [0; IPC_BUFFER_SIZE];
    let encoded_message = bincode::serialize(&ipc_message).unwrap();
    conn.write_all(&encoded_message)?;
    let mut buff_reader = BufReader::new(conn.borrow_mut());
    let _ = buff_reader.read(buffer.borrow_mut())?;
    Ok(bincode::deserialize::<IpcMessage>(buffer.borrow_mut()).unwrap())
}
