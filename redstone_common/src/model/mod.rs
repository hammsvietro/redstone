use std::{borrow::Borrow, fmt::Display};

use crate::web::api::ApiErrorResponse;

pub mod api;
pub mod backup;
pub mod config;
pub mod fs_tree;
pub mod ipc;
pub mod tcp;
pub mod track;

#[derive(Debug)]
pub enum RedstoneError {
    ApiError(ApiErrorResponse),
    BaseError(String),
    ConnectionTimeout,
    CronParseError(String),
    FolderOrFileNotFound(String),
    HttpError(String),
    IOError(String),
    NoHomeDir,
    SerdeError(String),
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

impl From<reqwest::Error> for RedstoneError {
    fn from(error: reqwest::Error) -> Self {
        RedstoneError::HttpError(error.to_string())
    }
}

impl Display for RedstoneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error: String = match self {
            Self::ApiError(error) => format_api_error(error),
            Self::BaseError(error) => error.to_owned(),
            Self::ConnectionTimeout => String::from("Connection timed out."),
            Self::CronParseError(cron) => format!("Couldn't parse cron string: {cron}"),
            Self::IOError(reason) => format!("{reason}"),
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
        };
        write!(f, "{}", error)
    }
}

fn format_api_error(error: &ApiErrorResponse) -> String {
    let mut error_string = String::from("");
    for (error, reasons) in error.errors.borrow().into_iter() {
        error_string += format!("{}:\n- {}\n", error, reasons.join("\n- ")).as_str();
    }
    error_string
}

pub type Result<T> = std::result::Result<T, RedstoneError>;
