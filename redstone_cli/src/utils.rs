use std::io::Write;

use interprocess::local_socket::LocalSocketStream;
use redstone_common::model::{
    ipc::{
        ConfirmationRequest, ConfirmationResponse, IpcMessage, IpcMessageResponse,
        IpcMessageResponseType,
    },
    DomainError, RedstoneError, Result,
};

use crate::ipc::socket::send_and_receive;

pub fn handle_confirmation_request(
    connection: &mut LocalSocketStream,
    request: &ConfirmationRequest,
) -> Result<IpcMessage> {
    print!("{}\nDo you wish to continue? [Y/n] ", request.message);
    std::io::stdout().flush()?;
    let mut buffer = String::new();
    let stdin = std::io::stdin();
    stdin.read_line(&mut buffer)?;
    let has_accepted = match buffer.to_lowercase().as_str() {
        "\n" | "y\n" | "yes\n" => true,
        "n\n" | "no\n" => false,
        _ => {
            println!("Couldn't parse, try again.");
            return handle_confirmation_request(connection, request);
        }
    };
    let response = IpcMessage::from(IpcMessageResponse {
        error: None,
        keep_connection: has_accepted,
        message: Some(IpcMessageResponseType::ConfirmationResponse(
            ConfirmationResponse { has_accepted },
        )),
    });

    if !has_accepted {
        send_and_receive(connection, response)?;
        return Err(RedstoneError::DomainError(
            DomainError::ConfirmationNotAccepted,
        ));
    }

    Ok(response)
}
