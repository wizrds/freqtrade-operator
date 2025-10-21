// SPDX-FileCopyrightText: 2025 Timothy Pogue
//
// SPDX-License-Identifier: ISC

use serde::Serialize;
use blake3::hash as blake3_hash;
use serde_json::Value;

/// Compute a hash for any serializable object
pub fn compute_object_hash<T>(object: &T) -> Result<String, Box<dyn std::error::Error>>
where
    T: Serialize,
{
    let value: Value = serde_json::from_str(&serde_json::to_string(object)?)?;
    let hash = blake3_hash(serde_json::to_string(&sort_json(value))?.as_bytes());

    Ok(hash.to_hex().to_string())
}

/// Recursively sort JSON objects
pub fn sort_json(value: Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(map.into_iter().map(|(k, v)| (k, sort_json(v))).collect()),
        Value::Array(arr) => Value::Array(arr.into_iter().map(sort_json).collect()),
        _ => value,
    }
}