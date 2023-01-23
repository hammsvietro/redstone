use std::borrow::BorrowMut;

use interprocess::local_socket::LocalSocketStream;

use redstone_common::{
    model::{
        api::{CloneRequest as ApiCloneRequest, Endpoints, CloneResponse},
        ipc::{clone::CloneRequest, IpcMessage, IpcMessageResponse, ConfirmationRequest},
        Result,
    },
    web::api::{jar::get_jar, RedstoneClient}, util::bytes_to_human_readable,
};
use reqwest::Method;

use crate::ipc::socket_loop::prompt_action_confirmation;

pub async fn handle_clone_msg(
    connection: &mut LocalSocketStream,
    clone_request: &mut CloneRequest,
) -> Result<IpcMessage> {
    println!("{:?}", clone_request);
    let client = RedstoneClient::new(get_jar()?);
    let request = &Some(ApiCloneRequest::new(clone_request.backup_name.clone()));
    let response = client
        .send(Method::POST, Endpoints::Clone.get_url(), request)
        .await?;

    let clone_response: CloneResponse = response.json().await?;

    let confirmation_request = ConfirmationRequest {
        message: get_confirmation_request_message(clone_response.total_bytes)
    };
    let confirmation_result =
        match prompt_action_confirmation(connection.borrow_mut(), confirmation_request).await {
            Ok(confirmation_response) => confirmation_response,
            Err(err) => return Err(err),
        };
    if !confirmation_result.has_accepted {
        return Ok(IpcMessageResponse {
            message: None,
            keep_connection: false,
            error: None,
            }.into()
        );
    }


    Ok(IpcMessageResponse {
        message: None,
        keep_connection: false,
        error: None,
    }.into())
}

fn get_confirmation_request_message(total_bytes: usize) -> String {
    let readable_bytes = bytes_to_human_readable(total_bytes);
    let mut message = format!("\nBy continuing, you will download {} of data", readable_bytes);
    message += "\nDo you wish to continue?";
    message
}
