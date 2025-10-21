use chrono::{DateTime, Utc};
use k8s_openapi::{
    api::core::v1::{Affinity, EnvVar, PodSecurityContext, ResourceRequirements, SecurityContext, Toleration, Volume, VolumeMount, Container},
    apimachinery::pkg::apis::meta::v1::ObjectMeta,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fmt::{Display, Formatter, Result as FmtResult}, collections::BTreeMap};
use schemars::JsonSchema;

use crate::crd::{hub::traits::Hub, hub::common::SecretItem, v1alpha1};

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Bot {
    pub metadata: ObjectMeta,
    pub spec: BotSpec,
    pub status: Option<BotStatus>,
}

impl Hub for Bot {}

impl From<v1alpha1::bot::Bot> for Bot {
    fn from(bot: v1alpha1::bot::Bot) -> Self {
        Bot {
            metadata: bot.metadata,
            spec: bot.spec.into(),
            status: bot.status.map(|status| status.into()),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BotSpec {
    pub exchange: String,
    #[serde(default = "default_database")]
    pub database: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<BTreeMap<String, Value>>,
    pub strategy: BotStrategySpec,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<BotModelSpec>,
    #[serde(default)]
    pub image: BotImageSpec,
    pub secrets: BotSecrets,
    #[serde(default)]
    pub api: BotApiSpec,
    #[serde(default)]
    pub service: BotServiceSpec,
    #[serde(default)]
    pub pvc: BotPvcSpec,
    #[serde(default)]
    pub deployment: BotDeploymentSpec,
}

impl From<v1alpha1::bot::BotSpec> for BotSpec {
    fn from(spec: v1alpha1::bot::BotSpec) -> Self {
        BotSpec {
            exchange: spec.exchange,
            database: spec.database,
            config: spec.config,
            strategy: spec.strategy.into(),
            model: spec.model.map(|model| model.into()),
            image: spec.image.into(),
            secrets: spec.secrets.into(),
            api: spec.api.into(),
            service: spec.service.into(),
            pvc: spec.pvc.into(),
            deployment: spec.deployment.into(),
        }
    }
}

fn default_database() -> String {
    "sqlite:///database.db".to_string()
}


#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BotStatus {
    pub phase: String,
    pub last_updated: Option<DateTime<Utc>>,
}

impl From<v1alpha1::bot::BotStatus> for BotStatus {
    fn from(status: v1alpha1::bot::BotStatus) -> Self {
        BotStatus {
            phase: status.phase,
            last_updated: status.last_updated,
        }
    }
}


#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct BotImageSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pull_policy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pull_secrets: Option<Vec<String>>,
}

impl Default for BotImageSpec {
    fn default() -> Self {
        BotImageSpec {
            repository: Some("freqtradeorg/freqtrade".to_string()),
            tag: Some("stable".to_string()),
            pull_policy: None,
            pull_secrets: None,
        }
    }
}

impl From<v1alpha1::bot::BotImageSpec> for BotImageSpec {
    fn from(spec: v1alpha1::bot::BotImageSpec) -> Self {
        BotImageSpec {
            repository: spec.repository,
            tag: spec.tag,
            pull_policy: spec.pull_policy,
            pull_secrets: spec.pull_secrets,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct BotSecrets {
    pub exchange: Option<ExchangeSecrets>,
    pub api: Option<ApiSecrets>,
    pub telegram: Option<TelegramSecrets>,
}

impl From<v1alpha1::bot::BotSecrets> for BotSecrets {
    fn from(secrets: v1alpha1::bot::BotSecrets) -> Self {
        BotSecrets {
            exchange: secrets.exchange.map(|exchange| exchange.into()),
            api: secrets.api.map(|api| api.into()),
            telegram: secrets.telegram.map(|telegram| telegram.into()),
        }
    }
}


#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ApiSecrets {
    pub username: Option<SecretItem>,
    pub password: Option<SecretItem>,
    pub ws_token: Option<SecretItem>,
    pub jwt_secret_key: Option<SecretItem>,
}

impl From<v1alpha1::bot::ApiSecrets> for ApiSecrets {
    fn from(secrets: v1alpha1::bot::ApiSecrets) -> Self {
        ApiSecrets {
            username: secrets.username.map(|username| username.into()),
            password: secrets.password.map(|password| password.into()),
            ws_token: secrets.ws_token.map(|ws_token| ws_token.into()),
            jwt_secret_key: secrets.jwt_secret_key.map(|jwt_secret_key| jwt_secret_key.into()),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct TelegramSecrets {
    pub token: Option<SecretItem>,
    pub chat_id: Option<String>,
}

impl From<v1alpha1::bot::TelegramSecrets> for TelegramSecrets {
    fn from(secrets: v1alpha1::bot::TelegramSecrets) -> Self {
        TelegramSecrets {
            token: secrets.token.map(|token| token.into()),
            chat_id: secrets.chat_id,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ExchangeSecrets {
    pub key: Option<SecretItem>,
    pub secret: Option<SecretItem>,
    pub password: Option<SecretItem>,
    pub uid: Option<SecretItem>,
}

impl From<v1alpha1::bot::ExchangeSecrets> for ExchangeSecrets {
    fn from(secrets: v1alpha1::bot::ExchangeSecrets) -> Self {
        ExchangeSecrets {
            key: secrets.key.map(|key| key.into()),
            secret: secrets.secret.map(|secret| secret.into()),
            password: secrets.password.map(|password| password.into()),
            uid: secrets.uid.map(|uid| uid.into()),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct BotStrategySpec {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_map_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

impl From<v1alpha1::bot::BotStrategySpec> for BotStrategySpec {
    fn from(spec: v1alpha1::bot::BotStrategySpec) -> Self {
        BotStrategySpec {
            name: spec.name,
            config_map_name: spec.config_map_name,
            source: spec.source,
        }
    }
}


#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct BotModelSpec {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_map_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

impl Default for BotModelSpec {
    fn default() -> Self {
        BotModelSpec {
            name: "LightGBMRegressor".to_string(),
            config_map_name: None,
            source: None,
        }
    }
}

impl From<v1alpha1::bot::BotModelSpec> for BotModelSpec {
    fn from(spec: v1alpha1::bot::BotModelSpec) -> Self {
        BotModelSpec {
            name: spec.name,
            config_map_name: spec.config_map_name,
            source: spec.source,
        }
    }
}


#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct BotApiSpec {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
}

impl Default for BotApiSpec {
    fn default() -> Self {
        BotApiSpec {
            enabled: true,
            host: "0.0.0.0".to_string(),
            port: 8080,
        }
    }
}

impl From<v1alpha1::bot::BotApiSpec> for BotApiSpec {
    fn from(spec: v1alpha1::bot::BotApiSpec) -> Self {
        BotApiSpec {
            enabled: spec.enabled,
            host: spec.host,
            port: spec.port,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct BotServiceSpec {
    pub service_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub labels: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub ports: Vec<BotServicePort>,
}

impl Default for BotServiceSpec {
    fn default() -> Self {
        BotServiceSpec {
            service_type: "ClusterIP".to_string(),
            annotations: None,
            labels: None,
            ports: vec![],
        }
    }
}

impl From<v1alpha1::bot::BotServiceSpec> for BotServiceSpec {
    fn from(spec: v1alpha1::bot::BotServiceSpec) -> Self {
        BotServiceSpec {
            service_type: spec.service_type,
            annotations: spec.annotations,
            labels: spec.labels,
            ports: spec.ports.into_iter().map(|port| port.into()).collect(),
        }
    }
}

impl BotServiceSpec {
    pub fn ensure_api_port(&mut self, api_port: u16) {
        if !self.ports.iter().any(|port| port.name == "api") {
            self.ports.push(BotServicePort {
                name: "api".to_string(),
                port: api_port,
                target_port: "api".to_string(),
            });
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct BotServicePort {
    pub name: String,
    pub port: u16,
    pub target_port: String,
}

impl From<v1alpha1::bot::BotServicePort> for BotServicePort {
    fn from(port: v1alpha1::bot::BotServicePort) -> Self {
        BotServicePort {
            name: port.name,
            port: port.port,
            target_port: port.target_port,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct BotPvcSpec {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_class: Option<String>,
    pub size: String,
}

impl Default for BotPvcSpec {
    fn default() -> Self {
        BotPvcSpec {
            enabled: true,
            annotations: None,
            labels: None,
            storage_class: None,
            size: "1Gi".to_string(),
        }
    }
}

impl From<v1alpha1::bot::BotPvcSpec> for BotPvcSpec {
    fn from(spec: v1alpha1::bot::BotPvcSpec) -> Self {
        BotPvcSpec {
            enabled: spec.enabled,
            annotations: spec.annotations,
            labels: spec.labels,
            storage_class: spec.storage_class,
            size: spec.size,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, JsonSchema)]
#[derive(Default)]
pub struct BotDeploymentSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub labels: Option<BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_selector: Option<BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourceRequirements>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub affinity: Option<Affinity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tolerations: Option<Vec<Toleration>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pod_security_context: Option<PodSecurityContext>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub security_context: Option<SecurityContext>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub containers: Vec<Container>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub init_containers: Vec<Container>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub volumes: Vec<Volume>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub volume_mounts: Vec<VolumeMount>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub env: Vec<EnvVar>,
}


impl From<v1alpha1::bot::BotDeploymentSpec> for BotDeploymentSpec {
    fn from(spec: v1alpha1::bot::BotDeploymentSpec) -> Self {
        BotDeploymentSpec {
            command: spec.command,
            annotations: spec.annotations,
            labels: spec.labels,
            node_selector: spec.node_selector,
            resources: spec.resources,
            affinity: spec.affinity,
            tolerations: spec.tolerations,
            pod_security_context: spec.pod_security_context,
            security_context: spec.security_context,
            containers: spec.containers,
            init_containers: spec.init_containers,
            volumes: spec.volumes,
            volume_mounts: spec.volume_mounts,
            env: spec.env,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum BotPhase {
    // The bot is pending
    Pending,
    // The bot is running
    Running,
    // The bot is errored
    Error,
    // The bot is being deleted
    Deleting,
}

impl Display for BotPhase {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            BotPhase::Pending => write!(f, "pending"),
            BotPhase::Running => write!(f, "running"),
            BotPhase::Error => write!(f, "error"),
            BotPhase::Deleting => write!(f, "deleting"),
        }
    }
}

impl From<v1alpha1::bot::BotPhase> for BotPhase {
    fn from(phase: v1alpha1::bot::BotPhase) -> Self {
        match phase {
            v1alpha1::bot::BotPhase::Pending => BotPhase::Pending,
            v1alpha1::bot::BotPhase::Running => BotPhase::Running,
            v1alpha1::bot::BotPhase::Error => BotPhase::Error,
            v1alpha1::bot::BotPhase::Deleting => BotPhase::Deleting,
        }
    }
}
