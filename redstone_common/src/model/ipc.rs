use super::track::{TrackMessageResponse, TrackRequest};
use serde::{Deserialize, Serialize};

///
/// BASE
///

#[derive(Debug, Serialize, Deserialize)]
pub enum IpcMessage {
    Request(IpcMessageRequest),
    Response(IpcMessageResponse),
}

///
/// REQUEST
///

#[derive(Debug, Serialize, Deserialize)]
pub struct IpcMessageRequest {
    pub message: IpcMessageRequestType,
}

impl IpcMessageRequest {
    pub fn is_confirmation(&self) -> bool {
        match self.message {
            IpcMessageRequestType::ConfirmationRequest(_) => true,
            _ => false,
        }
    }

    pub fn get_confirmation(self) -> ConfirmationRequest {
        match self.message {
            IpcMessageRequestType::ConfirmationRequest(req) => req,
            _ => panic!("Tried to 'get_confirmation' but wrapped object is of another type."),
        }
    }
}

impl Into<IpcMessage> for IpcMessageRequest {
    fn into(self) -> IpcMessage {
        IpcMessage::Request(self)
    }
}

impl From<IpcMessage> for IpcMessageRequest {
    fn from(msg: IpcMessage) -> Self {
        if let IpcMessage::Request(req) = msg {
            return req;
        }
        panic!("Tried to unwrap 'IpcMessageRequest' from 'IpcMessage'")
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum IpcMessageRequestType {
    TrackRequest(TrackRequest),
    ConfirmationRequest(ConfirmationRequest),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfirmationRequest {
    pub message: String,
}

impl From<IpcMessage> for ConfirmationRequest {
    fn from(ipc_message: IpcMessage) -> Self {
        if let IpcMessage::Request(ipc_req) = ipc_message {
            return ipc_req.get_confirmation();
        }
        panic!("Tried to 'get_confirmation' but wrapped object is of another type.");
    }
}

impl From<ConfirmationRequest> for IpcMessage {
    fn from(req: ConfirmationRequest) -> Self {
        IpcMessage::Request(IpcMessageRequest {
            message: IpcMessageRequestType::ConfirmationRequest(req),
        })
    }
}

///
/// RESPONSE
///

#[derive(Debug, Serialize, Deserialize)]
pub struct IpcMessageResponse {
    pub message: Option<IpcMessageResponseType>,
    pub keep_connection: bool,
    pub error: Option<String>,
}

impl From<IpcMessageResponse> for IpcMessage {
    fn from(res: IpcMessageResponse) -> Self {
        IpcMessage::Response(res)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IpcMessageResponseType {
    TrackMessageResponse(TrackMessageResponse),
    ConfirmationResponse(ConfirmationResponse),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfirmationResponse {
    pub has_accepted: bool,
}

impl From<ConfirmationResponse> for IpcMessage {
    fn from(res: ConfirmationResponse) -> Self {
        IpcMessage::Response(IpcMessageResponse {
            keep_connection: res.has_accepted,
            error: None,
            message: Some(IpcMessageResponseType::ConfirmationResponse(res)),
        })
    }
}

impl From<IpcMessage> for ConfirmationResponse {
    fn from(msg: IpcMessage) -> Self {
        if let IpcMessage::Response(res) = msg {
            if let Some(IpcMessageResponseType::ConfirmationResponse(confirmation)) = res.message {
                return confirmation;
            }
        }
        panic!("Tried to extract 'ConfirmationResponse' from 'IpcMessage' but wrapped object is of another type.");
    }
}

impl IpcMessageResponse {
    pub fn shutdown_connection(&self) -> bool {
        self.error.is_some() || !self.keep_connection
    }
}
