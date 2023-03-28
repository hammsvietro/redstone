use std::borrow::{Borrow, BorrowMut};

use interprocess::local_socket::{LocalSocketListener, LocalSocketStream};
use redstone_common::{
    constants::IPC_SOCKET_PATH,
    ipc::send,
    model::{
        ipc::{IpcMessage, IpcMessageRequest, IpcMessageRequestType, IpcMessageResponse},
        Result,
    },
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    ipc::{
        clone::handle_clone_msg, handle_error, read_message_until_complete_or_timeout,
        track::handle_track_msg,
    },
    scheduler::UpdateJob,
};

use super::{pull::handle_pull_msg, push::handle_push_msg};

pub async fn run_socket_loop(new_job_sender: UnboundedSender<UpdateJob>) -> Result<()> {
    let listener = LocalSocketListener::bind(IPC_SOCKET_PATH)?;
    println!("Listening on {IPC_SOCKET_PATH}");
    for mut conn in listener.incoming().filter_map(handle_error) {
        let new_job_sender = new_job_sender.clone();
        tokio::spawn(async move { handle_connection(conn.borrow_mut(), new_job_sender).await });
    }
    Ok(())
}

async fn handle_connection(
    connection: &mut LocalSocketStream,
    _new_job_sender: UnboundedSender<UpdateJob>,
) -> Result<()> {
    loop {
        let ipc_message = match read_message_until_complete_or_timeout(
            connection.borrow_mut(),
            std::time::Duration::from_secs(5),
        )
        .await
        {
            Ok(message) => message,
            Err(err) => {
                let _ = send(
                    connection.borrow_mut(),
                    &IpcMessage::Response(IpcMessageResponse {
                        message: None,
                        error: Some(err.clone()),
                        keep_connection: false,
                    }),
                );
                return Err(err);
            }
        };
        let message = IpcMessageRequest::from(ipc_message);
        let result_msg = handle_message(connection.borrow_mut(), message.message)
            .await
            .unwrap_or_else(|err| {
                IpcMessage::Response(IpcMessageResponse {
                    message: None,
                    keep_connection: false,
                    error: Some(err),
                })
            });

        if let Err(err) = send(connection.borrow_mut(), result_msg.borrow()) {
            eprintln!("{err}");
            break;
        }
        if let IpcMessage::Response(res) = result_msg {
            if res.shutdown_connection() {
                break;
            }
        }
    }
    // let new_job = updatejob {
    //     backup_id: 4,
    //     cron_expr: string::from("*/2 * * * * * *")
    // };
    // new_job_sender.send(new_job).unwrap();
    println!("Closing socket");
    Ok(())
}

async fn handle_message(
    connection: &mut LocalSocketStream,
    message: IpcMessageRequestType,
) -> Result<IpcMessage> {
    match message {
        IpcMessageRequestType::TrackRequest(mut track_request) => {
            handle_track_msg(connection.borrow_mut(), &mut track_request).await
        }
        IpcMessageRequestType::CloneRequest(mut clone_request) => {
            handle_clone_msg(connection.borrow_mut(), &mut clone_request).await
        }
        IpcMessageRequestType::PushRequest(mut push_request) => {
            handle_push_msg(connection.borrow_mut(), &mut push_request).await
        }
        IpcMessageRequestType::PullRequest(mut pull_request) => {
            handle_pull_msg(connection.borrow_mut(), &mut pull_request).await
        }
        _ => unreachable!(),
    }
}
