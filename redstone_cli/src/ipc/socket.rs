use interprocess::local_socket::LocalSocketStream;
use redstone_common::{
    constants::{IPC_BUFFER_SIZE, IPC_SOCKET_PATH},
    model::{ipc::IpcMessage, Result},
};
use std::{
    borrow::BorrowMut,
    io::{prelude::*, BufReader},
};

pub fn stablish_connection() -> Result<LocalSocketStream> {
    Ok(LocalSocketStream::connect(IPC_SOCKET_PATH)?)
}

pub fn send_and_receive(
    conn: &mut LocalSocketStream,
    ipc_message: IpcMessage,
) -> Result<IpcMessage> {
    send(conn, ipc_message)?;
    receive(conn)
}

fn send(conn: &mut LocalSocketStream, ipc_message: IpcMessage) -> Result<()> {
    let encoded_message = bincode::serialize(&ipc_message)?;
    Ok(conn.write_all(&encoded_message)?)
}

fn receive(conn: &mut LocalSocketStream) -> Result<IpcMessage> {
    let mut buffer = [0; IPC_BUFFER_SIZE];
    let mut buff_reader = BufReader::new(conn.borrow_mut());
    let _ = buff_reader.read(buffer.borrow_mut())?;
    Ok(bincode::deserialize::<IpcMessage>(buffer.borrow_mut())?)
}
