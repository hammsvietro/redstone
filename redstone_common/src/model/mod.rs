use std::fmt::Display;

pub mod api;
pub mod backup;
pub mod config;
pub mod fs_tree;
pub mod ipc;
pub mod track;
pub mod tcp;

#[derive(Debug)]
pub enum RedstoneError {
    ConnectionTimeout,
    CronParseError(String),
    IOError(String),
    FolderOrFileNotFound(String),
    NoHomeDir,
    SerdeError(String),
    Unauthorized,
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

impl From<bson::ser::Error> for RedstoneError {
    fn from(error: bson::ser::Error) -> Self {
        RedstoneError::SerdeError(error.to_string())
    }
}

impl Display for RedstoneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error: String = match self {
            Self::ConnectionTimeout => String::from("Connection timed out."),
            Self::CronParseError(cron) => format!("Couldn't parse cron string: {cron}"),
            Self::IOError(reason) => format!("{reason}"),
            Self::FolderOrFileNotFound(path) => format!("Couldn't open a file/folder: {path}"),
            Self::NoHomeDir => String::from("Couldn't find your home directory."),
            Self::Unauthorized => {
                String::from("Unauthorized, check if you're logged in correctly.")
            }
            Self::SerdeError(error) => {
                format!("An error occoured while serializing or serializing data:\n{error}")
            }
        };
        write!(f, "{}", error)
    }
}

pub type Result<T> = std::result::Result<T, RedstoneError>;
