use redstone_common::model::{api::File, Result};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::mpsc::UnboundedSender,
};

pub async fn send_files(
    files: Vec<File>,
    upload_token: String,
    progress_emitter: UnboundedSender<u64>,
) -> Result<()> {
    let stream = TcpStream::connect("127.0.0.1:8000").await?;
    let mut stream = BufReader::new(stream);
    let packet = "ping";
    send_message(&mut stream, packet).await.unwrap();
    let received_packet = receive_message(&mut stream)
        .await
        .expect("Coudn't read message.");
    println!("{}", received_packet);
    Ok(())
}

fn get_message_size_in_bytes(message: &str) -> [u8; 4] {
    (message.as_bytes().len() as u32).to_be_bytes()
}

async fn send_message(stream: &mut BufReader<TcpStream>, packet: &str) -> Result<()> {
    let packet_size = get_message_size_in_bytes(packet);
    Ok(stream
        .write_all(&[&packet_size, packet.as_bytes()].concat())
        .await?)
}

async fn receive_message(stream: &mut BufReader<TcpStream>) -> Result<String> {
    let mut incoming_packet_buf: [u8; 4] = [0; 4];
    stream.read_exact(&mut incoming_packet_buf).await?;
    let incoming_packet_size = u32::from_be_bytes(incoming_packet_buf);
    let mut buffer = vec![0; incoming_packet_size as usize];
    stream.read_exact(&mut buffer).await?;
    Ok(String::from_utf8(buffer).unwrap())
}
