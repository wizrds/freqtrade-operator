// SPDX-FileCopyrightText: 2025 Timothy Pogue
//
// SPDX-License-Identifier: ISC

use anyhow::Error;
use serde::{Deserialize, Serialize};
use std::result;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct APIError {
    pub code: u16,
    pub message: String,
}

impl APIError {
    pub fn new(code: u16, message: String) -> Self {
        Self { code, message }
    }

    pub fn unexpected_error(message: &str) -> Self {
        Self {
            code: 50000,
            message: message.to_string(),
        }
    }

    pub fn invalid_content_type(content_type: &str) -> Self {
        Self {
            code: 40001,
            message: format!("Invalid content type: {}", content_type),
        }
    }

    pub fn invalid_data_format(message: &str) -> Self {
        Self {
            code: 40002,
            message: message.to_string(),
        }
    }

    pub fn not_implemented() -> Self {
        Self {
            code: 50001,
            message: "Not implemented".to_string(),
        }
    }
}

impl From<Error> for APIError {
    fn from(error: Error) -> Self {
        APIError::unexpected_error(&error.to_string())
    }
}

pub type APIResult<T> = result::Result<T, APIError>;