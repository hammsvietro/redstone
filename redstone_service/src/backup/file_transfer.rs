use redstone_common::model::api::File;
use tokio::{sync::mpsc::UnboundedSender, net::TcpStream, io::{BufReader, AsyncWriteExt, AsyncBufReadExt}};

pub async fn send_files(files: Vec<File>, upload_token: String, progress_emitter: UnboundedSender<u64>) -> std::io::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:8000").await?;
    let mut stream = BufReader::new(stream);
    let message = "ping";
    let size = get_message_size_in_bytes(message);

    let message = [&size, message.as_bytes()].concat();
    stream.write_all(&message).await?;
    let mut buf = String::new();
    stream.read_line(&mut buf).await?;
    for _ in 0..4 {
        buf.remove(0);
    }
    println!("{}", buf);
    Ok(())
}

fn get_message_size_in_bytes(message: &str) -> [u8; 4] {
    (message.as_bytes().len() as u32).to_be_bytes()
}
