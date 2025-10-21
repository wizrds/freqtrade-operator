use serde_json::Value;
use kube::core::DynamicObject;

use crate::admission::{error::{AdmissionResult, AdmissionError}, utils::check_key_exists};


fn validate_bot_v1alpha1(spec: &Value) -> AdmissionResult<()> {
    // These keys are reserved and cannot be used in the bot config
    // as they are injected by the operator, or not supported
    // by the operator.
    const RESERVED_CONFIG_KEYS: &[&str] = &[
        "config.add_config_files",
        "config.recursive_strategy_search",
        "config.strategy_path",
        "config.strategy",
        "config.bot_name",
        "config.db_url",
        "config.api_server.enabled",
        "config.api_server.listen_ip_address",
        "config.api_server.listen_port",
        "config.api_server.jwt_secret_key",
        "config.api_server.username",
        "config.api_server.password",
        "config.api_server.ws_token",
        "config.telegram.token",
        "config.telegram.chat_id",
        "config.exchange.name",
        "config.exchange.key",
        "config.exchange.secret",
        "config.exchange.password",
        "config.freqai.enabled",
    ];
    const RESERVED_ENV_VARS: &[&str] = &[
        "FREQTRADE__STRATEGY",
        "FREQTRADE__STRATEGY_PATH",
        "FREQTRADE__DB_URL",
        "FREQTRADE__BOT_NAME",
        "FREQTRADE__API_SERVER__ENABLED",
        "FREQTRADE__API_SERVER__LISTEN_IP_ADDRESS",
        "FREQTRADE__API_SERVER__LISTEN_PORT",
        "FREQTRADE__API_SERVER__USERNAME",
        "FREQTRADE__API_SERVER__PASSWORD",
        "FREQTRADE__API_SERVER__JWT_SECRET_KEY",
        "FREQTRADE__API_SERVER__WS_TOKEN",
        "FREQTRADE__EXCHANGE__NAME",
        "FREQTRADE__EXCHANGE__KEY",
        "FREQTRADE__EXCHANGE__SECRET",
        "FREQTRADE__EXCHANGE__PASSWORD",
        "FREQTRADE__EXCHANGE__UID",
        "FREQTRADE__TELEGRAM__TOKEN",
        "FREQTRADE__TELEGRAM__CHAT_ID",
    ];

    for key in RESERVED_CONFIG_KEYS {
        if check_key_exists(spec, key) {
            return Err(AdmissionError::ValidationError(format!("config key `{}` is reserved", key)));
        }
    }

    for key in RESERVED_ENV_VARS {
        if check_key_exists(spec, key) {
            return Err(AdmissionError::ValidationError(format!("env var `{}` is reserved", key)));
        }
    }

    Ok(())
}

pub fn validate_bot_crd(payload: &DynamicObject) -> AdmissionResult<()> {
    let payload_types = payload.types.clone().unwrap();

    if payload_types.kind != "Bot" {
        return Err(AdmissionError::InvalidKind(payload_types.kind, "Bot".to_string()));
    }

    let version = payload_types
        .api_version
        .split("/")
        .last()
        .unwrap_or(&payload_types.api_version);
    let json_spec = serde_json::to_value(payload.data.get("spec")).unwrap();

    match version {
        "v1alpha1" => validate_bot_v1alpha1(&json_spec),
        _ => Err(AdmissionError::InvalidVersion(version.to_string(), "Bot".to_string())),
    }
}
