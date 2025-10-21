use kube::CustomResource;
use k8s_openapi::api::core::v1::{
    Affinity, Toleration, PodSecurityContext, ResourceRequirements, SecurityContext,
    Container, Volume, VolumeMount, EnvVar,
};
use std::{fmt::{Display, Formatter, Result as FmtResult}, string::ToString, collections::BTreeMap};
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::crd::v1alpha1::common::SecretItem;

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
#[kube(
    kind = "Bot",
    group = "freqtrade.io",
    version = "v1alpha1",
    status = "BotStatus",
    doc = "Bot is a specification for a Freqtrade bot running in a Kubernetes cluster.",
    derive = "PartialEq",
    printcolumn = r#"{"name":"Phase", "type":"string", "description":"Current phase of the resource", "jsonPath":".status.phase"}"#,
    printcolumn = r#"{"name":"Exchange", "type":"string", "description":"Exchange the bot is trading on", "jsonPath":".spec.exchange"}"#,
    printcolumn = r#"{"name":"Last Updated", "type":"date", "description":"Last time the resource was updated", "jsonPath":".status.lastUpdated"}"#,
    namespaced
)]
#[serde(rename_all = "camelCase")]
pub struct BotSpec {
    /// Name of the exchange the bot is trading on.
    pub exchange: String,
    #[serde(default = "default_database")]
    /// Database URL to use for the bot
    pub database: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(schema_with = "any_nested_object_schema")]
    /// Configuration for the bot.
    pub config: Option<BTreeMap<String, Value>>,
    /// Strategy to use for the bot
    pub strategy: BotStrategySpec,
    /// Model to use for the bot
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<BotModelSpec>,
    #[serde(default)]
    /// Image to use for the bot
    pub image: BotImageSpec,
    #[serde(default)]
    /// Secrets to use for the bot
    pub secrets: BotSecrets,
    #[serde(default)]
    /// API configuration for the bot
    pub api: BotApiSpec,
    #[serde(default)]
    /// Service resource additional configuration
    pub service: BotServiceSpec,
    #[serde(default)]
    /// PersistentVolumeClaim resource configuration
    pub pvc: BotPvcSpec,
    #[serde(default)]
    /// Deployment resource additional configuration
    pub deployment: BotDeploymentSpec,
}

fn default_database() -> String {
    "sqlite:///database.db".to_string()
}

fn any_nested_object_schema(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
    serde_json::from_value(serde_json::json!({
        "type": "object",
        "additionalProperties": {
            "x-kubernetes-preserve-unknown-fields": true,
        },
        "x-kubernetes-preserve-unknown-fields": true,
    }))
    .unwrap()
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BotStatus {
    pub phase: String,
    pub last_updated: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct BotImageSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Repository to pull the image from
    pub repository: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Tag to pull
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Image pull policy
    pub pull_policy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Secrets to use for pulling the image
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

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
#[derive(Default)]
pub struct BotSecrets {
    /// Exchange secrets to use for the bot
    pub exchange: Option<ExchangeSecrets>,
    /// API secrets to use for the bot
    pub api: Option<ApiSecrets>,
    /// Telegram secrets to use for the bot
    pub telegram: Option<TelegramSecrets>,
}


#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
#[derive(Default)]
pub struct ApiSecrets {
    /// API username used for authentication
    pub username: Option<SecretItem>,
    /// API password used for authentication
    pub password: Option<SecretItem>,
    /// API websocket token used for consumers to connect to producers
    pub ws_token: Option<SecretItem>,
    /// Secret JWT key used to sign JWT tokens
    pub jwt_secret_key: Option<SecretItem>,
}


#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
#[derive(Default)]
pub struct TelegramSecrets {
    /// The Telegram token
    pub token: Option<SecretItem>,
    /// The Telegram chat ID to send messages to
    pub chat_id: Option<String>,
}


#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
#[derive(Default)]
pub struct ExchangeSecrets {
    /// The exchange key
    pub key: Option<SecretItem>,
    /// The exchange secret
    pub secret: Option<SecretItem>,
    /// The exchange password
    pub password: Option<SecretItem>,
    /// The exchange userid
    pub uid: Option<SecretItem>,
}


#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BotStrategySpec {
    /// The strategy class name to use
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The ConfigMap to pull the source from, containing the `strategy.py` key
    pub config_map_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The source code for the strategy
    pub source: Option<String>,
}


#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct BotModelSpec {
    /// The model class name to use
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The ConfigMap to pull the source from, containing the `model.py` key
    pub config_map_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The source code for the model
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


#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct BotApiSpec {
    /// Whether the API is enabled or not
    pub enabled: bool,
    /// The host to bind the API to
    pub host: String,
    /// The port to bind the API to
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

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BotServiceSpec {
    /// The service type to use, defaults to `ClusterIP`
    pub service_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Additional annotations to add to the service
    pub annotations: Option<BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Additional labels to add to the service
    pub labels: Option<BTreeMap<String, String>>,
    #[serde(default)]
    /// Additonal ports to expose on the service
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

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BotServicePort {
    /// The name of the port
    pub name: String,
    /// The port to expose
    pub port: u16,
    /// The target port to forward to
    pub target_port: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct BotPvcSpec {
    /// Whether the PVC is enabled or not
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Additional annotations to add to the PVC
    pub annotations: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Additional labels to add to the PVC
    pub labels: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The storage class to use for the PVC, defaults to the cluster's default storage class
    pub storage_class: Option<String>,
    /// The size of the PVC, defaults to `1Gi`
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

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
#[derive(Default)]
pub struct BotDeploymentSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// A custom command to run in the container, overrides the default command
    pub command: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Additional annotations to add to the deployment
    pub annotations: Option<BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Additional labels to add to the deployment
    pub labels: Option<BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Node selector to use for the deployment
    pub node_selector: Option<BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// The compute resource constraints and requests for the deployment
    pub resources: Option<ResourceRequirements>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// The affinity rules for the deployment
    pub affinity: Option<Affinity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// The tolerations for the deployment
    pub tolerations: Option<Vec<Toleration>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// The pod's security context
    pub pod_security_context: Option<PodSecurityContext>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// The container's security context
    pub security_context: Option<SecurityContext>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Additional containers to add to the deployment
    pub containers: Vec<Container>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Additional init containers to add to the deployment
    pub init_containers: Vec<Container>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Additional volumes to add to the deployment
    pub volumes: Vec<Volume>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Additional volume mounts to add to the pod's main container
    pub volume_mounts: Vec<VolumeMount>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Additional environment variables to add to the deployment
    pub env: Vec<EnvVar>,
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