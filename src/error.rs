use std::{
    convert::Infallible, io, num::TryFromIntError, result::Result as StdResult, str::Utf8Error,
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    WalkDir(#[from] async_walkdir::Error),
    // #[error("Invalid duration: {0}")]
    // InvalidDuration(String),
    // #[error("Invalid memory size: {0}")]
    // InvalidMemorySize(String),
    // #[error("Invalid judge type: {0}")]
    // InvalidJudgeType(String),
    // #[error("Invalid task type: {0}")]
    // InvalidTaskType(String),
    // #[error("Invalid resource limits: {0}")]
    // InvalidResourceLimits(String),
    #[error("invalid value")]
    InvalidValue(#[from] TryFromIntError),
    #[error("invalid filename")]
    InvalidFilename(#[from] Utf8Error),
    // #[error("Invalid score: {0}")]
    // InvalidScore(u32),
}

pub type Result<T, E = Error> = StdResult<T, E>;

impl From<Infallible> for Error {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}
