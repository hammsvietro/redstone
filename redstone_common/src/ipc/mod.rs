use std::{
    borrow::BorrowMut,
    io::{BufReader, Read, Write},
};

use interprocess::local_socket::LocalSocketStream;

use crate::model::{ipc::IpcMessage, Result};

pub fn send_and_receive(
    conn: &mut LocalSocketStream,
    ipc_message: &IpcMessage,
) -> Result<IpcMessage> {
    send(conn, ipc_message)?;
    receive(conn)
}

pub fn send(conn: &mut LocalSocketStream, ipc_message: &IpcMessage) -> Result<()> {
    let encoded_message = bincode::serialize(ipc_message)?;
    let message_size = get_message_size_in_bytes(&encoded_message);
    Ok(conn.write_all(&[&message_size, encoded_message.as_slice()].concat())?)
}

pub fn receive(conn: &mut LocalSocketStream) -> Result<IpcMessage> {
    let mut incoming_packet_buf: [u8; 4] = [0; 4];
    let mut buff_reader = BufReader::new(conn.borrow_mut());
    buff_reader.read_exact(&mut incoming_packet_buf)?;
    let incoming_packet_size = u32::from_be_bytes(incoming_packet_buf);
    let mut buffer = vec![0; incoming_packet_size as usize];
    buff_reader.read_exact(buffer.borrow_mut())?;
    Ok(bincode::deserialize::<IpcMessage>(buffer.borrow_mut())?)
}

fn get_message_size_in_bytes(message: &[u8]) -> [u8; 4] {
    (message.len() as u32).to_be_bytes()
}
