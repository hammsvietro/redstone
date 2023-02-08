use std::{
    borrow::{Borrow, BorrowMut},
    io::{self, prelude::*, BufReader},
};

use interprocess::local_socket::{LocalSocketListener, LocalSocketStream};
use redstone_common::{
    constants::{IPC_BUFFER_SIZE, IPC_SOCKET_PATH},
    model::{
        ipc::{
            ConfirmationRequest, ConfirmationResponse, IpcMessage, IpcMessageRequest,
            IpcMessageRequestType, IpcMessageResponse,
        },
        Result,
    },
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    ipc::{clone::handle_clone_msg, track::handle_track_msg},
    scheduler::UpdateJob,
};

use super::push::handle_push_msg;

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
                let _ = send_message(
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

        if let Err(err) = send_message(connection.borrow_mut(), result_msg.borrow()) {
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

fn try_reading(
    connection: &mut LocalSocketStream,
    buffer: &mut [u8],
) -> redstone_common::model::Result<usize> {
    let mut buff_reader = BufReader::new(connection.borrow_mut());
    Ok(buff_reader.read(buffer.borrow_mut())?)
}

fn send_message(connection: &mut LocalSocketStream, message: &IpcMessage) -> Result<()> {
    let message: &[u8] = &bincode::serialize(message.borrow()).unwrap();
    Ok(connection.write_all(message)?)
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
        _ => unreachable!(),
    }
}

fn continue_reading_message(error: &bincode::ErrorKind) -> bool {
    match error {
        bincode::ErrorKind::Io(io_error) => io_error.kind() == io::ErrorKind::UnexpectedEof,
        _ => false,
    }
}

async fn read_message_until_complete_or_timeout(
    connection: &mut LocalSocketStream,
    timeout: std::time::Duration,
) -> Result<IpcMessage> {
    let mut buffer = [0; IPC_BUFFER_SIZE];
    let mut request_buffer: Vec<u8> = Vec::new();
    let mut last_received_data_time = std::time::SystemTime::now();
    let mut size: usize;
    loop {
        size = try_reading(connection.borrow_mut(), buffer.borrow_mut())?;
        if size > 0 {
            request_buffer.extend_from_slice(&buffer[0..size]);
            if size == IPC_BUFFER_SIZE {
                continue;
            }
            last_received_data_time = std::time::SystemTime::now();
            match bincode::deserialize::<IpcMessage>(&request_buffer) {
                Ok(message) => return Ok(message),
                Err(err) => {
                    if continue_reading_message(err.borrow()) || size == IPC_BUFFER_SIZE {
                        continue;
                    }
                    return Err(redstone_common::model::RedstoneError::SerdeError(
                        err.to_string(),
                    ));
                }
            };
        } else {
            let duration = std::time::SystemTime::now()
                .duration_since(last_received_data_time)
                .unwrap();
            if duration > timeout {
                return Err(redstone_common::model::RedstoneError::ConnectionTimeout);
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    }
}

fn handle_error(
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
    send_message(socket.borrow_mut(), &IpcMessage::from(confirmation_request)).unwrap();
    let ipc_message = read_message_until_complete_or_timeout(
        socket.borrow_mut(),
        std::time::Duration::from_secs(60 * 2),
    )
    .await?;
    Ok(ConfirmationResponse::from(ipc_message))
}
