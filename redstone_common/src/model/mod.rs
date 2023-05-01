use std::fmt::Display;

use serde::{Deserialize, Serialize};
use tokio::task::JoinError;

use crate::web::api::ApiErrorResponse;

pub mod api;
pub mod backup;
pub mod config;
pub mod fs_tree;
pub mod ipc;
pub mod tcp;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RedstoneError {
    ApiError(ApiErrorResponse),
    ArgumentError(ArgumentError),
    BaseError(String),
    ConnectionTimeout,
    CronParseError(String),
    DomainError(DomainError),
    FolderOrFileNotFound(String),
    HttpError(String),
    IOError(String),
    NoHomeDir,
    SerdeError(String),
    TokioError(String),
    Unauthorized,
}

impl From<std::string::String> for RedstoneError {
    fn from(str: std::string::String) -> Self {
        RedstoneError::BaseError(str)
    }
}

impl From<ApiErrorResponse> for RedstoneError {
    fn from(error: ApiErrorResponse) -> Self {
        RedstoneError::ApiError(error)
    }
}

impl From<JoinError> for RedstoneError {
    fn from(error: JoinError) -> Self {
        RedstoneError::TokioError(error.to_string())
    }
}

impl From<std::io::Error> for RedstoneError {
    fn from(error: std::io::Error) -> Self {
        RedstoneError::IOError(error.to_string())
    }
}

impl From<Box<bincode::ErrorKind>> for RedstoneError {
    fn from(error_kind: Box<bincode::ErrorKind>) -> Self {
        RedstoneError::SerdeError(error_kind.to_string())
    }
}

impl From<bson::de::Error> for RedstoneError {
    fn from(error: bson::de::Error) -> Self {
        RedstoneError::SerdeError(error.to_string())
    }
}

impl From<bson::ser::Error> for RedstoneError {
    fn from(error: bson::ser::Error) -> Self {
        RedstoneError::SerdeError(error.to_string())
    }
}

impl From<tokio::sync::mpsc::error::SendError<u64>> for RedstoneError {
    fn from(error: tokio::sync::mpsc::error::SendError<u64>) -> Self {
        RedstoneError::TokioError(error.to_string())
    }
}

impl From<reqwest::Error> for RedstoneError {
    fn from(error: reqwest::Error) -> Self {
        RedstoneError::HttpError(error.to_string())
    }
}

impl From<ignore::Error> for RedstoneError {
    fn from(error: ignore::Error) -> Self {
        RedstoneError::IOError(error.to_string())
    }
}

impl Display for RedstoneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error: String = match self {
            Self::ApiError(error) => error.stringified_errors.to_owned(),
            Self::BaseError(error) => error.to_owned(),
            Self::ArgumentError(error) => error.to_string(),
            Self::DomainError(error) => error.to_string(),
            Self::ConnectionTimeout => String::from("Connection timed out."),
            Self::CronParseError(cron) => format!("Couldn't parse cron string: {cron}"),
            Self::IOError(reason) => reason.to_string(),
            Self::FolderOrFileNotFound(path) => format!("Couldn't open a file/folder: {path}"),
            Self::NoHomeDir => String::from("Couldn't find your home directory."),
            Self::Unauthorized => {
                String::from("Unauthorized, check if you're logged in correctly.")
            }
            Self::HttpError(error) => {
                format!("An error happened while doing an http request:\n{error}")
            }
            Self::SerdeError(error) => {
                format!("An error occoured while serializing or serializing data:\n{error}")
            }
            Self::TokioError(error) => error.to_owned(),
        };
        write!(f, "{error}")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ArgumentError {
    InvalidPath(String),
    PathCannotBeAFile(String),
}

impl Display for ArgumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error: String = match self {
            Self::InvalidPath(path) => format!("Path \"{path}\" is not valid."),
            Self::PathCannotBeAFile(path) => format!("Path \"{path}\" cannot be a file."),
        };
        write!(f, "{error}")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DomainError {
    BackupAlreadyExists(String),
    BackupDoesntExist(String),
    NoServerConfigFound,
    NotAuthenticated,
    NotInLatestUpdate,
    AlreadyInLatestUpdate,
    NoChanges,
    ConfirmationNotAccepted,
    ErrorDurringProgressEmition,
}

impl Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error: String = match self {
            Self::BackupAlreadyExists(path) => {
                format!("Backup already exists in the provided directory: \"{path}\"")
            }
            Self::BackupDoesntExist(path) => {
                format!("Backup doesn't exist in the provided directory: \"{path}\"")
            }
            Self::NoChanges => "No changes to push".into(),
            Self::AlreadyInLatestUpdate => "\
            \nAlready in latest update.\
            "
            .into(),
            Self::NotInLatestUpdate => "\
            \nThere's a newer version of this backup.\
            \n\nPlease, pull the changes with \"redstone pull\" before pushing.\
            \nBeware, pulling data might overwrite your current changes\
            "
            .into(),
            Self::ConfirmationNotAccepted => "".into(),
            Self::ErrorDurringProgressEmition => "".into(),
            Self::NotAuthenticated => "Not authenticated, run redstone auth to authenticate".into(),
            Self::NoServerConfigFound => {
                "No server configuration found. Use the command: redstone set-server-address".into()
            }
        };
        write!(f, "{error}")
    }
}

pub type Result<T> = std::result::Result<T, RedstoneError>;
