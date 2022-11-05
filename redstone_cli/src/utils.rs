use std::io::Write;

use redstone_common::model::{
    ipc::{ConfirmationRequest, ConfirmationResponse, IpcMessageResponse, IpcMessageResponseType},
    Result,
};

pub fn handle_confirmation_request(request: &ConfirmationRequest) -> Result<IpcMessageResponse> {
    print!("{} [Y/n] ", request.message);
    std::io::stdout().flush()?;
    let mut buffer = String::new();
    let stdin = std::io::stdin();
    stdin.read_line(&mut buffer)?;
    let has_accepted = match buffer.to_lowercase().as_str() {
        "\n" | "y\n" => true,
        "n\n" => false,
        _ => {
            println!("Couldn't parse, try again.");
            return handle_confirmation_request(request);
        }
    };
    Ok(IpcMessageResponse {
        error: None,
        keep_connection: has_accepted,
        message: Some(IpcMessageResponseType::ConfirmationResponse(
            ConfirmationResponse { has_accepted },
        )),
    })
}
