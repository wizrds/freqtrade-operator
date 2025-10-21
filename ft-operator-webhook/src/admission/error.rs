use thiserror::Error;
use std::result::Result;

#[derive(Error, Debug)]
pub enum AdmissionError {
    #[error("invalid kind: {0} expected {1}")]
    InvalidKind(String, String),
    #[error("invalid version: {0} for {1}")]
    InvalidVersion(String, String),
    #[error("validation error: {0}")]
    ValidationError(String),
}

pub type AdmissionResult<T> = Result<T, AdmissionError>;