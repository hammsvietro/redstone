use std::{borrow::BorrowMut, path::PathBuf};

use interprocess::local_socket::LocalSocketStream;
use redstone_common::{
    constants::IPC_SOCKET_PATH,
    ipc::{receive, send, send_and_receive},
    model::{
        fs_tree::FSTree,
        ipc::{
            ConfirmationRequest, ConfirmationResponse, FileActionProgress, IpcMessage,
            IpcMessageRequest, IpcMessageRequestType,
        },
        DomainError, RedstoneError, Result,
    },
};
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    task::spawn_blocking,
};

pub mod clone;
pub mod pull;
pub mod push;
pub mod socket_loop;
pub mod track;

pub fn assert_socket_is_available() {
    let _ = std::fs::remove_file(IPC_SOCKET_PATH);
}

pub async fn read_message_until_complete_or_timeout(
    connection: &mut LocalSocketStream,
    timeout: std::time::Duration,
) -> Result<IpcMessage> {
    tokio::select! {
        message = async {receive(connection)} => {
            message
        }
        _ = tokio::time::sleep(timeout) => {
            Err(redstone_common::model::RedstoneError::ConnectionTimeout)
        }
    }
}

pub fn handle_error(
    conn: std::result::Result<LocalSocketStream, std::io::Error>,
) -> Option<LocalSocketStream> {
    match conn {
        Ok(val) => Some(val),
        Err(error) => {
            eprintln!("Incoming connection failed: {error}");
            None
        }
    }
}

pub async fn prompt_action_confirmation(
    socket: &mut LocalSocketStream,
    confirmation_request: ConfirmationRequest,
) -> Result<ConfirmationResponse> {
    send(socket.borrow_mut(), &IpcMessage::from(confirmation_request))?;
    let ipc_message = read_message_until_complete_or_timeout(
        socket.borrow_mut(),
        std::time::Duration::from_secs(60 * 2),
    )
    .await?;
    Ok(ConfirmationResponse::from(ipc_message))
}

pub async fn send_progress(
    conn: &mut LocalSocketStream,
    progress_receiver: &mut UnboundedReceiver<FileActionProgress>,
) -> Result<()> {
    while let Some(progress) = progress_receiver.recv().await {
        let response = send_and_receive(
            conn,
            &IpcMessage::Request(IpcMessageRequest {
                message: IpcMessageRequestType::FileActionProgress(progress),
            }),
        )?;
        match response {
            IpcMessage::Response(res) if res.keep_connection && res.error.is_none() => continue,
            _ => {
                return Err(RedstoneError::DomainError(
                    DomainError::ErrorDurringProgressEmition,
                ))
            }
        };
    }
    Ok(())
}

pub fn progress_sender_factory(
    progress_sender: &UnboundedSender<FileActionProgress>,
) -> impl Fn(FileActionProgress) + '_ {
    |progress| {
        let _ = progress_sender.send(progress);
    }
}

pub async fn build_fs_tree_with_progress(
    connection: &mut LocalSocketStream,
    root: PathBuf,
) -> Result<FSTree> {
    let (tx, mut rx) = unbounded_channel::<FileActionProgress>();
    let (send_progress_result, fs_tree) = tokio::join!(
        send_progress(connection.borrow_mut(), &mut rx),
        spawn_blocking(move || {
            let progress_sender_factory = &progress_sender_factory(&tx);
            FSTree::build(root, Some(progress_sender_factory))
        })
    );
    send_progress_result?;
    fs_tree?
}
