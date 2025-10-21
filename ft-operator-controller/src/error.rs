// SPDX-FileCopyrightText: 2025 Timothy Pogue
//
// SPDX-License-Identifier: ISC

use std::result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ControllerError {
    #[error("failed to create client: {0}")]
    KubeError(#[from] kube::Error),
    #[error("missing object key: {0}")]
    MissingObjectKeyError(&'static str),
    #[error("finalizer error: {0}")]
    FinalizerError(String),
    #[error("unknown error: {0}")]
    UnknownError(String),
}

pub type Result<T> = result::Result<T, ControllerError>;