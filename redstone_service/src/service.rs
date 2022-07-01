//! Definition of the service loop

use tokio::sync::mpsc;

use crate::ipc::socket_loop::run_socket_loop;
use crate::scheduler::{run_upload_jobs, UpdateJob};

pub async fn run_service() -> std::io::Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel::<UpdateJob>();
    let result = tokio::try_join!(run_socket_loop(tx), run_upload_jobs(&mut rx));

    if let Err(err) = result {
        println!("An error happened: {:?}", err);
    }

    Ok(())
}
