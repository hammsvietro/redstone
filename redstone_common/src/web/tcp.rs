use serde::de::DeserializeOwned;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

use crate::model::Result;

pub async fn send_message(stream: &mut BufReader<TcpStream>, packet: &[u8]) -> Result<()> {
    let packet_size = get_message_size_in_bytes(packet);
    Ok(stream.write_all(&[&packet_size, packet].concat()).await?)
}

pub async fn receive_message<T: DeserializeOwned>(stream: &mut BufReader<TcpStream>) -> Result<T> {
    let mut incoming_packet_buf: [u8; 4] = [0; 4];
    stream.read_exact(&mut incoming_packet_buf).await?;
    let incoming_packet_size = u32::from_be_bytes(incoming_packet_buf);
    let mut buffer = vec![0; incoming_packet_size as usize];
    stream.read_exact(&mut buffer).await?;
    Ok(bson::from_slice(&buffer)?)
}

fn get_message_size_in_bytes(message: &[u8]) -> [u8; 4] {
    (message.len() as u32).to_be_bytes()
}
