use std::path::PathBuf;
use std::str::Utf8Error;
use std::string::FromUtf8Error;
use std::{env, io};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HyprMountError {
    #[error("File operation failed: {0}")]
    Io(#[from] io::Error),

    #[error("File reading error: {failure_msg}")]
    FileReadingError { failure_msg: String },

    #[error("Failed to parse configuration: {0}")]
    Parse(String),

    #[error("Invalid path: {path} is not a directory")]
    InvalidDir { path: PathBuf },

    #[error("Failed conversion from slice to UTF-8 string: {0}")]
    ToStrConvFail(#[from] Utf8Error),

    #[error("Configuration parsing failed: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Mounting error: {failure_msg}")]
    MountError { failure_msg: String },

    #[error("UDisksCtl error: {err_msg}")]
    UDiskCtlError { err_msg: String },

    #[error("Could not find home directory: {0}")]
    HomePath(#[from] env::VarError),

    #[error("Failed to convert executable path to string (probably invalid UTF-8)")]
    ExePath(),

    #[error("Failed to convert Vec of UTF-8 to String: {0}")]
    FromUtf8ToStringFail(#[from] FromUtf8Error),
}
