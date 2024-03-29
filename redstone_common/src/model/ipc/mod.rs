pub mod clone;
pub mod pull;
pub mod push;
pub mod track;

use std::fmt::Debug;

use self::push::PushRequest;
use self::track::TrackRequest;
use self::{clone::CloneRequest, pull::PullRequest};
use super::RedstoneError;

use serde::{Deserialize, Serialize};

///
/// BASE
///

#[derive(Debug, Serialize, Deserialize)]
pub enum IpcMessage {
    Request(IpcMessageRequest),
    Response(IpcMessageResponse),
}

impl IpcMessage {
    pub fn is_request(&self) -> bool {
        matches!(self, Self::Request(_))
    }

    pub fn is_confirmation_request(&self) -> bool {
        match self {
            Self::Request(request) => request.is_confirmation(),
            _ => false,
        }
    }

    pub fn is_file_progress(&self) -> bool {
        match self {
            IpcMessage::Request(request) => request.is_progress(),
            _ => false,
        }
    }

    pub fn is_response(&self) -> bool {
        matches!(self, Self::Response(_))
    }

    pub fn has_errors(&self) -> bool {
        match self {
            Self::Response(response) => response.error.is_some(),
            _ => false,
        }
    }
}

///
/// REQUEST
///

#[derive(Debug, Serialize, Deserialize)]
pub struct IpcMessageRequest {
    pub message: IpcMessageRequestType,
}

impl IpcMessageRequest {
    pub fn is_progress(&self) -> bool {
        matches!(self.message, IpcMessageRequestType::FileActionProgress(_))
    }
    pub fn is_confirmation(&self) -> bool {
        matches!(self.message, IpcMessageRequestType::ConfirmationRequest(_))
    }

    pub fn get_confirmation(self) -> ConfirmationRequest {
        match self.message {
            IpcMessageRequestType::ConfirmationRequest(req) => req,
            _ => panic!("Tried to 'get_confirmation' but wrapped object is of another type."),
        }
    }

    pub fn get_progress(self) -> FileActionProgress {
        match self.message {
            IpcMessageRequestType::FileActionProgress(req) => req,
            _ => panic!("Tried to 'get_progress' but wrapped object is of another type."),
        }
    }
}

impl From<IpcMessageRequest> for IpcMessage {
    fn from(val: IpcMessageRequest) -> Self {
        IpcMessage::Request(val)
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
    CloneRequest(CloneRequest),
    ConfirmationRequest(ConfirmationRequest),
    PushRequest(PushRequest),
    PullRequest(PullRequest),
    TrackRequest(TrackRequest),
    FileActionProgress(FileActionProgress),
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
    pub error: Option<RedstoneError>,
}

impl From<IpcMessageResponse> for IpcMessage {
    fn from(res: IpcMessageResponse) -> Self {
        IpcMessage::Response(res)
    }
}

impl From<IpcMessage> for IpcMessageResponse {
    fn from(message: IpcMessage) -> Self {
        if let IpcMessage::Response(res) = message {
            return res;
        }
        panic!("Tried to extract 'IpcMessageResponse' from 'IpcMessage' but wrapped object is of another type.");
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IpcMessageResponseType {
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

///
/// TransferProgress
///
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct FileActionProgress {
    pub current_file_name: String,
    pub progress: u64,
    pub total: u64,
    pub operation: FileAction,
}

impl From<IpcMessage> for FileActionProgress {
    fn from(ipc_message: IpcMessage) -> Self {
        if let IpcMessage::Request(ipc_req) = ipc_message {
            return ipc_req.get_progress();
        }
        panic!("Tried to 'get_confirmation' but wrapped object is of another type.");
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
pub enum FileAction {
    #[default]
    Hash,
    Download,
    Upload,
}

impl FileAction {
    pub fn get_progress_bar_message(&self, file_name: &str) -> String {
        match self {
            FileAction::Download => format!("Downloading {file_name}"),
            FileAction::Upload => format!("Uploading {file_name}"),
            FileAction::Hash => format!("Hashing {file_name}"),
        }
    }

    pub fn get_action_message(&self) -> &'static str {
        match self {
            FileAction::Download => "Files downloaded",
            FileAction::Upload => "Files uploaded",
            FileAction::Hash => "Files hashed",
        }
    }
}
