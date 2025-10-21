// SPDX-FileCopyrightText: 2025 Timothy Pogue
//
// SPDX-License-Identifier: ISC

use std::path::Path;
use serde::{Serialize, Deserialize};
use figment::{Figment, Error, providers::{Format, Json, Yaml, Env, Serialized}};

use crate::constant::ENV_PREFIX;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(unused)]
#[derive(Default)]
pub struct AppConfig {
    #[serde(default)]
    pub controller: ControllerConfig,
    #[serde(default)]
    pub webhook: WebhookConfig,
}


#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(unused)]
pub struct ControllerConfig {
    #[serde(default)]
    pub default_image_repo: String,
    #[serde(default)]
    pub default_image_tag: String,
}

impl Default for ControllerConfig {
    fn default() -> Self {
        ControllerConfig {
            default_image_repo: "freqtradeorg/freqtrade".to_string(),
            default_image_tag: "stable".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(unused)]
pub struct WebhookConfig {
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub port: u16,
    #[serde(default)]
    pub tls: TLSConfig,
}

impl Default for WebhookConfig {
    fn default() -> Self {
        WebhookConfig {
            host: "0.0.0.0".to_string(),
            port: 8443,
            tls: TLSConfig::default(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(unused)]
pub struct TLSConfig {
    #[serde(default)]
    pub cert_file: String,
    #[serde(default)]
    pub key_file: String,
}

impl Default for TLSConfig {
    fn default() -> Self {
        TLSConfig {
            cert_file: "/etc/ssl/certs/tls.crt".to_string(),
            key_file: "/etc/ssl/certs/tls.key".to_string(),
        }
    }
}

pub struct AppConfigBuilder {
    figment: Figment,
}

impl AppConfigBuilder {
    pub fn with_file(&mut self, path: &str) -> &mut Self {
        let extension = Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default();

        self.figment = match extension {
            "json" => self.figment.clone().merge(Json::file(path).nested()),
            "yaml" | "yml" => self.figment.clone().merge(Yaml::file(path).nested()),
            _ => self.figment.clone(),
        };
        self
    }

    pub fn with_env(&mut self) -> &mut Self {
        self.figment = self.figment.clone().merge(Env::prefixed(&format!("{}__", ENV_PREFIX)).split("__"));
        self
    }

    pub fn with_override_option(&mut self, key: &str, value: Option<&str>) -> &mut Self {
        if let Some(value) = value {
            self.figment = self.figment.clone().merge(Serialized::default(key, value));
        }
        self
    }

    pub fn build(&self) -> Result<AppConfig, Error> {
        self.figment.extract()
    }
}

impl Default for AppConfigBuilder {
    fn default() -> Self {
        AppConfigBuilder {
            figment: Figment::from(Serialized::defaults(AppConfig::default()))
        }
    }
}