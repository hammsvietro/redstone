use std::path::PathBuf;

use redstone_common::model::{api::File, Result, tcp::{TcpMessage, FileUploadMessageFactory}, RedstoneError};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::mpsc::UnboundedSender,
};

pub async fn send_files(
    files: Vec<File>,
    upload_token: String,
    root_folder: PathBuf,
    _progress_emitter: UnboundedSender<u64>,
) -> Result<()> {
    let stream = TcpStream::connect("127.0.0.1:8000").await?;
    let mut stream = BufReader::new(stream);
    for file in files {
        println!("Uploading {} file", file.path);
        let mut file_upload_message = FileUploadMessageFactory::new(upload_token.clone(), file, root_folder.clone());
        while file_upload_message.has_data_to_fetch() {
            let packet = file_upload_message.get_tcp_payload()?;
            send_message(&mut stream, &packet).await?;
            let response = receive_message(&mut stream).await?;
            if response != String::from("ACK\n") {
                // TODO: return correct error and send abort message
                println!("NOT ACK");
                return Err(RedstoneError::NoHomeDir);
            }
        }
    }
    Ok(())
}

fn get_message_size_in_bytes(message: &[u8]) -> [u8; 4] {
    (message.len() as u32).to_be_bytes()
}

async fn send_message(stream: &mut BufReader<TcpStream>, packet: &[u8]) -> Result<()> {
    let packet_size = get_message_size_in_bytes(packet);
    Ok(stream
        .write_all(&[&packet_size, packet].concat())
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
