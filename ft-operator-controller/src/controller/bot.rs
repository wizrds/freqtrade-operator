use kube::{
    api::{Api, Patch, PatchParams, ResourceExt, ObjectMeta},
    runtime::{
        controller::{Action, Controller},
        finalizer::{finalizer, Event as Finalizer},
        watcher,
    },
};
use k8s_openapi::{api::apps::v1::{Deployment, DeploymentSpec, DeploymentStatus}, apimachinery::pkg::api::resource::Quantity};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::LabelSelector;
use k8s_openapi::api::core::v1::{
    Service, ServiceSpec, ServicePort, ConfigMap, PersistentVolumeClaim, Secret,
    PodSpec, PodTemplateSpec, Container, EnvVar, EnvVarSource, ConfigMapVolumeSource,
    ContainerPort, VolumeMount, Volume, PersistentVolumeClaimSpec, VolumeResourceRequirements,
    PersistentVolumeClaimVolumeSource, KeyToPath, SecretKeySelector, LocalObjectReference,
};
use k8s_openapi::apimachinery::pkg::{
    apis::meta::v1::OwnerReference,
    util::intstr::IntOrString
};
use std::sync::Arc;
use std::string::ToString;
use std::collections::BTreeMap;
use chrono::Utc;
use tokio::time::Duration;
use serde_json::json;

use ft_operator_common::config::AppConfig;
use ft_operator_common::telemetry::info;
use ft_operator_common::utils::compute_object_hash;

use crate::controller::{context::Context, traits::{FromHub, ResourceDrift}, utils::{apply, delete, rollout, patch, FIELD_MANAGER}};
use crate::crd::{NamespacedCustomResource, hub::bot::{Bot, BotPhase, BotStatus}, hub::common::SecretItem};
use crate::error::{Result, ControllerError};


pub static FINALIZER: &str = "bots.finalizers.freqtrade.io";
pub static CONFIG_HASH_ANNOTATION: &str = "bots.freqtrade.io/config-hash";

impl From<DeploymentStatus> for BotPhase {
    /// Convert a DeploymentStatus to a BotPhase
    /// This function is responsible for converting a DeploymentStatus to a BotPhase.
    /// 
    /// # Arguments
    /// * `status` - The DeploymentStatus to convert
    /// 
    /// # Returns
    /// The BotPhase
    fn from(status: DeploymentStatus) -> Self {
        match status.conditions {
            Some(conditions) => {
                if conditions.iter().any(|c| c.type_ == "Progressing" && c.status == "False") {
                    BotPhase::Error
                } else if conditions.iter().any(|c| c.type_ == "Available" && c.status == "True") {
                    BotPhase::Running
                } else {
                    BotPhase::Pending // Default to Pending if no conditions match
                }
            }
            None => BotPhase::Pending, // Default to Pending if no conditions are available
        }
    }
}

impl FromHub<Bot> for ConfigMap {
    /// Create a ConfigMap resource from a Bot Hub
    /// 
    /// This function is responsible for creating a ConfigMap resource from a Bot Hub.
    /// 
    /// # Arguments
    /// * `bot` - The Bot CRD to create the ConfigMap resource from
    /// * `name` - The name of the ConfigMap resource
    /// * `namespace` - The namespace of the ConfigMap resource
    /// * `owner_ref` - The owner reference for the ConfigMap resource
    /// * `config` - The application configuration
    /// 
    /// # Returns
    /// The ConfigMap resource
    fn from_hub(bot: &Bot, name: &str, namespace: &str, owner_ref: OwnerReference, _config: &AppConfig) -> Self {
        let config_data = bot.spec.config.clone();
        let strategy = bot.spec.strategy.clone();
        let model = bot.spec.model.clone();

        ConfigMap {
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                namespace: Some(namespace.to_string()),
                owner_references: Some(vec![owner_ref]),
                ..Default::default()
            },
            data: Some(BTreeMap::from([
                ("config.json".to_string(), serde_json::to_string(&config_data).unwrap_or_default()),
            ])
            .into_iter()
            .chain(
                strategy.config_map_name
                    .is_none()
                    .then(|| (
                        "strategy.py".to_string(),
                        strategy.source.unwrap_or_default(),
                    ))
            )
            .chain(
                model
                    .as_ref()
                    .and_then(|m| m.source.clone())
                    .map_or_else(
                        || None,
                        |source| Some(("model.py".to_string(), source))
                    )
            )
            .collect()),
            ..Default::default()
        }
    }
}

impl ResourceDrift<Bot> for ConfigMap {
    /// Determine if the ConfigMap resource has drifted from another ConfigMap resource
    /// derived from the Bot CRD
    /// 
    /// # Arguments
    /// * `other` - The other ConfigMap resource to compare against
    /// 
    /// # Returns
    /// Whether the ConfigMap resource has drifted from the other ConfigMap resource
    fn has_drifted(&self, other: &Self) -> bool {
        // We just compare the data field which should have only 1-2 keys, with strings as values
        self.data != other.data
    }
}

impl FromHub<Bot> for PersistentVolumeClaim {
    /// Create a PersistentVolumeClaim resource from a Bot Hub
    /// 
    /// This function is responsible for creating a PersistentVolumeClaim resource from a Bot Hub.
    /// 
    /// # Arguments
    /// * `bot` - The Bot CRD to create the PersistentVolumeClaim resource from
    /// * `name` - The name of the PersistentVolumeClaim resource
    /// * `namespace` - The namespace of the PersistentVolumeClaim resource
    /// * `owner_ref` - The owner reference for the PersistentVolumeClaim resource
    /// * `config` - The application configuration
    /// 
    /// # Returns
    /// The PersistentVolumeClaim resource
    fn from_hub(bot: &Bot, name: &str, namespace: &str, owner_ref: OwnerReference, _config: &AppConfig) -> Self {
        let pvc = bot.spec.pvc.clone();

        PersistentVolumeClaim {
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                namespace: Some(namespace.to_string()),
                owner_references: Some(vec![owner_ref]),
                annotations: pvc.annotations.clone(),
                labels: pvc.labels.clone(),
                ..Default::default()
            },
            spec: Some(PersistentVolumeClaimSpec {
                access_modes: Some(vec!["ReadWriteOnce".to_string()]),
                resources: Some(VolumeResourceRequirements {
                    requests: Some(BTreeMap::from([("storage".to_string(), Quantity(pvc.size.clone()))])),
                    ..Default::default()
                }),
                storage_class_name: pvc.storage_class,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

impl ResourceDrift<Bot> for PersistentVolumeClaim {
    /// Determine if the PersistentVolumeClaim resource has drifted from another PersistentVolumeClaim resource
    /// derived from the Bot CRD
    /// 
    /// # Arguments
    /// * `other` - The other PersistentVolumeClaim resource to compare against
    /// 
    /// # Returns
    /// Whether the PersistentVolumeClaim resource has drifted from the other PersistentVolumeClaim resource
    fn has_drifted(&self, other: &Self) -> bool {
        match (&self.spec, &other.spec) {
            // If both specs are Some, compare the storage class name, and resources
            (Some(spec), Some(other_spec)) => {
                // If either of the specs have a storage_class_name of None, and if not they do not equal,
                // or if the resources are different, then there is drift
                (spec.storage_class_name.is_some() && other_spec.storage_class_name.is_some()
                    && spec.storage_class_name != other_spec.storage_class_name)
                    || spec.resources != other_spec.resources
            },
            // If one of the specs is None, then there is drift
            (None, Some(_)) | (Some(_), None) => true,
            // If both specs are None, then there is no drift
            (None, None) => false,
        }
    }
}

impl FromHub<Bot> for Deployment {
    /// Create a Deployment resource from a Bot Hub
    /// 
    /// This function is responsible for creating a Deployment resource from a Bot Hub.
    /// 
    /// # Arguments
    /// * `bot` - The Bot CRD to create the Deployment resource from
    /// * `name` - The name of the Deployment resource
    /// * `namespace` - The namespace of the Deployment resource
    /// * `owner_ref` - The owner reference for the Deployment resource
    /// * `config` - The application configuration
    /// 
    /// # Returns
    /// The Deployment resource
    fn from_hub(bot: &Bot, name: &str, namespace: &str, owner_ref: OwnerReference, config: &AppConfig) -> Self {
        let image = bot.spec.image.clone();
        let api = bot.spec.api.clone();
        let strategy = bot.spec.strategy.clone();
        let model = bot.spec.model.clone();
        let pvc = bot.spec.pvc.clone();
        let deployment = bot.spec.deployment.clone();
        let secrets = bot.spec.secrets.clone();

        let image_repo = image.repository.unwrap_or(config.controller.default_image_repo.clone());
        let image_tag = image.tag.unwrap_or(config.controller.default_image_tag.clone());

        let identifying_labels = BTreeMap::from([
            ("freqtrade.io/bot-name".to_string(), name.to_string()),
            ("app.kubernetes.io/name".to_string(), name.to_string()),
            ("app.kubernetes.io/instance".to_string(), name.to_string()),
        ]);
        let metadata_labels = BTreeMap::from([
            ("app.kubernetes.io/component".to_string(), "bot".to_string()),
            ("app.kubernetes.io/part-of".to_string(), "freqtrade".to_string()),
            ("app.kubernetes.io/managed-by".to_string(), "freqtrade-operator".to_string()),
        ]);

        let default_command: Vec<String> = vec![
            "freqtrade".to_string(),
            "trade".to_string(),
            "--config".to_string(),
            "/etc/freqtrade/config.json".to_string(),
        ]
        .into_iter()
        .chain(
            model
            .as_ref()
            .map(|m| {
                vec![
                    "--freqaimodel".to_string(),
                    m.name.clone().to_string(),
                ]
            })
            .into_iter()
            .flatten()
        )
        .collect();

        Deployment {
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                namespace: Some(namespace.to_string()),
                owner_references: Some(vec![owner_ref]),
                annotations: deployment.annotations.clone(),
                labels: deployment.labels.clone()
                    .map_or(
                        Some(identifying_labels.clone()
                            .into_iter()
                            .chain(metadata_labels.clone())
                            .collect()
                        ),
                        |mut labels| {
                            labels.extend(identifying_labels.clone());
                            labels.extend(metadata_labels.clone());
                            Some(labels)
                    }),
                ..Default::default()
            },
            spec: Some(DeploymentSpec {
                // The Bot instance will always have only 1 replica, as Freqtrade can not inherently
                // scale horizontally.
                replicas: Some(1),
                selector: LabelSelector {
                    match_labels: Some(identifying_labels.clone()),
                    ..Default::default()
                },
                template: PodTemplateSpec {
                    metadata: Some(ObjectMeta {
                        annotations: deployment.annotations.clone(),
                        labels: deployment.labels.clone()
                            .map_or(
                                Some(identifying_labels.clone()
                                    .into_iter()
                                    .chain(metadata_labels.clone())
                                    .collect()
                                ),
                                |mut labels| {
                                    labels.extend(identifying_labels.clone());
                                    labels.extend(metadata_labels.clone());
                                    Some(labels)
                                }
                            ),
                        ..Default::default()
                    }),
                    spec: Some(PodSpec {
                        image_pull_secrets: image.pull_secrets.as_ref().map(|secrets| secrets.iter().map(|secret| {
                            LocalObjectReference {
                                name: secret.clone(),
                            }
                        }).collect()),
                        containers: vec![
                            Container {
                                name: name.to_string(),
                                image: Some(format!("{}:{}", image_repo, image_tag)),
                                image_pull_policy: image.pull_policy,
                                command: Some(match &deployment.command {
                                    Some(cmd) => cmd
                                        .iter()
                                        .flat_map(|part| match part.as_str() {
                                            "$CMD" => default_command.clone(),
                                            _ => vec![part.clone()],
                                        })
                                        .collect(),
                                    None => default_command.clone(),
                                }),
                                env: Some(vec![
                                    // Environment variables
                                    create_env_var("FREQTRADE__STRATEGY", Some(strategy.name)),
                                    create_env_var("FREQTRADE__STRATEGY_PATH", Some("/etc/freqtrade".to_string())),
                                    create_env_var("FREQTRADE__FREQAIMODEL_PATH", Some("/etc/freqtrade".to_string())),
                                    create_env_var("FREQTRADE__DB_URL", Some(bot.spec.database.to_string())),
                                    create_env_var("FREQTRADE__BOT_NAME", Some(name.to_string())),
                                    create_env_var("FREQTRADE__API_SERVER__ENABLED", Some(api.enabled.to_string())),
                                    create_env_var("FREQTRADE__API_SERVER__LISTEN_IP_ADDRESS", Some(api.host.to_string())),
                                    create_env_var("FREQTRADE__API_SERVER__LISTEN_PORT", Some(api.port.to_string())),
                                    create_env_var("FREQTRADE__EXCHANGE__NAME", Some(bot.spec.exchange.to_string())),
                                    secrets.telegram.as_ref().map_or_else(
                                        || create_env_var("FREQTRADE__TELEGRAM__CHAT_ID", None),
                                        |t| create_env_var("FREQTRADE__TELEGRAM__CHAT_ID", Some(t.chat_id.clone().unwrap_or_default()))
                                    ),
                                    // Secret-based environment variables
                                    secrets.api.as_ref().map_or_else(
                                        || create_env_var("FREQTRADE__API_SERVER__USERNAME", None),
                                        |a| create_secret_env_var("FREQTRADE__API_SERVER__USERNAME", &a.username)
                                    ),
                                    secrets.api.as_ref().map_or_else(
                                        || create_env_var("FREQTRADE__API_SERVER__PASSWORD", None),
                                        |a| create_secret_env_var("FREQTRADE__API_SERVER__PASSWORD", &a.password)
                                    ),
                                    secrets.api.as_ref().map_or_else(
                                        || create_env_var("FREQTRADE__API_SERVER__WS_TOKEN", None),
                                        |a| create_secret_env_var("FREQTRADE__API_SERVER__WS_TOKEN", &a.ws_token)
                                    ),
                                    secrets.api.as_ref().map_or_else(
                                        || create_env_var("FREQTRADE__API_SERVER__JWT_SECRET_KEY", None),
                                        |a| create_secret_env_var("FREQTRADE__API_SERVER__JWT_SECRET_KEY", &a.jwt_secret_key)
                                    ),
                                    secrets.telegram.as_ref().map_or_else(
                                        || create_env_var("FREQTRADE__TELEGRAM__TOKEN", None),
                                        |t| create_secret_env_var("FREQTRADE__TELEGRAM__TOKEN", &t.token)
                                    ),
                                    secrets.exchange.as_ref().map_or_else(
                                        || create_env_var("FREQTRADE__EXCHANGE__KEY", None),
                                        |e| create_secret_env_var("FREQTRADE__EXCHANGE__KEY", &e.key)
                                    ),
                                    secrets.exchange.as_ref().map_or_else(
                                        || create_env_var("FREQTRADE__EXCHANGE__SECRET", None),
                                        |e| create_secret_env_var("FREQTRADE__EXCHANGE__SECRET", &e.secret)
                                    ),
                                    secrets.exchange.as_ref().map_or_else(
                                        || create_env_var("FREQTRADE__EXCHANGE__PASSWORD", None),
                                        |e| create_secret_env_var("FREQTRADE__EXCHANGE__PASSWORD", &e.password)
                                    ),
                                    secrets.exchange.as_ref().map_or_else(
                                        || create_env_var("FREQTRADE__EXCHANGE__UID", None),
                                        |e| create_secret_env_var("FREQTRADE__EXCHANGE__UID", &e.uid)
                                    ),
                                ]
                                .into_iter()
                                .chain(
                                    model
                                        .as_ref()
                                        .map(|_| create_env_var("FREQTRADE__FREQAI__ENABLED", Some("true".to_string())))
                                )
                                .chain(deployment.env.clone().into_iter())
                                .collect()),
                                ports: Some(vec![
                                    ContainerPort {
                                        container_port: api.port as i32,
                                        name: Some("api".to_string()),
                                        ..Default::default()
                                    },
                                ]),
                                volume_mounts: Some(vec![
                                    VolumeMount {
                                        name: "config".to_string(),
                                        mount_path: "/etc/freqtrade".to_string(),
                                        ..Default::default()
                                    },
                                ]
                                .into_iter()
                                .chain(deployment.volume_mounts.clone().into_iter())
                                .collect()),
                                ..Default::default()
                            },
                        ]
                        .into_iter()
                        .chain(deployment.containers.clone())
                        .collect(),
                        init_containers: match deployment.init_containers.is_empty() {
                            true => None,
                            false => Some(deployment.init_containers.clone()),
                        },
                        volumes: Some(
                            vec![
                                Volume {
                                    name: "config".to_string(),
                                    config_map: Some(ConfigMapVolumeSource {
                                        name: name.to_string(),
                                        items: Some(
                                            vec![
                                                KeyToPath {
                                                    key: "config.json".to_string(),
                                                    path: "config.json".to_string(),
                                                    ..Default::default()
                                                },
                                            ]
                                            .into_iter()
                                            .chain(
                                                strategy.config_map_name
                                                    .is_none()
                                                    .then(|| KeyToPath {
                                                        key: "strategy.py".to_string(),
                                                        path: "strategy.py".to_string(),
                                                        ..Default::default()
                                                    })
                                            )
                                            .chain(
                                                model
                                                    .as_ref()
                                                    .filter(|m| m.source.is_some() && m.config_map_name.is_none())
                                                    .map(|_| KeyToPath {
                                                        key: "model.py".to_string(),
                                                        path: "model.py".to_string(),
                                                        ..Default::default()
                                                    })
                                            )
                                            .collect(),
                                        ),
                                        ..Default::default()
                                    }),
                                    ..Default::default()
                                },
                            ]
                            .into_iter()
                            .chain(
                                pvc.enabled
                                    .then(|| Volume {
                                        name: "user-data".to_string(),
                                        persistent_volume_claim: Some(PersistentVolumeClaimVolumeSource {
                                            claim_name: name.to_string(),
                                            ..Default::default()
                                        }),
                                        ..Default::default()
                                    })
                            )
                            .chain(
                                strategy.config_map_name
                                    .clone()
                                    .map(|_| Volume {
                                        name: "strategy".to_string(),
                                        config_map: Some(ConfigMapVolumeSource {
                                            name: strategy.config_map_name.unwrap(),
                                            ..Default::default()
                                        }),
                                        ..Default::default()
                                    })
                            )
                            .chain(
                                model
                                    .as_ref()
                                    .and_then(|m| m.config_map_name.clone())
                                    .map(|name| Volume {
                                        name: "model".to_string(),
                                        config_map: Some(ConfigMapVolumeSource {
                                            name,
                                            ..Default::default()
                                        }),
                                        ..Default::default()
                                    })
                            )
                            .chain(deployment.volumes.clone())
                            .collect(),
                        ),
                        ..Default::default()
                    }),
                },
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

impl ResourceDrift<Bot> for Deployment {
    /// Determine if two Deployment resources have drifted.
    ///
    /// This function compares the current Deployment resource (`self`) with another Deployment
    /// resource (`other`) to check if any relevant fields have changed, indicating drift.
    ///
    /// # Arguments
    /// * `other` - The other Deployment resource to compare against.
    ///
    /// # Returns
    /// `true` if there is a drift, `false` otherwise.
    fn has_drifted(&self, other: &Self) -> bool {
        // Compare spec.replicas
        if self.spec
            .as_ref()
            .and_then(|spec| spec.replicas)
            != other.spec
                .as_ref()
                .and_then(|spec| spec.replicas)
        {
            return true;
        }

        // Compare container configuration (image, env, command, ports, etc.)
        let self_containers = self
            .spec
            .as_ref()
            .and_then(|spec| spec.template.spec.as_ref())
            .map(|pod_spec| pod_spec.containers.clone())
            .unwrap_or_default();

        let other_containers = other
            .spec
            .as_ref()
            .and_then(|spec| spec.template.spec.as_ref())
            .map(|pod_spec| pod_spec.containers.clone())
            .unwrap_or_default();

        if self_containers.len() != other_containers.len() {
            return true;
        }

        for (self_container, other_container) in self_containers.iter().zip(&other_containers) {
            // Compare image, command, ports
            if self_container.image != other_container.image
                || self_container.command != other_container.command
                || compare_container_ports(self_container.ports.as_ref(), other_container.ports.as_ref())
            {
                return true;
            }

            // Compare image pull policy. If both have values, compare them. If one or both are None, there is no drift.
            if (self_container.image_pull_policy.is_some() && other_container.image_pull_policy.is_some())
                && self_container.image_pull_policy != other_container.image_pull_policy
            {
                return true;
            }

            // Compare environment variables
            if compare_env_vars(self_container.env.as_ref(), other_container.env.as_ref()) {
                return true;
            }

            // Compare volume mounts
            if self_container.volume_mounts != other_container.volume_mounts {
                return true;
            }

            // Compare container resources (CPU/Memory limits and requests). Unless both
            // sides have values, we consider them equal. If both sides do have values, then
            // we compare them.
            if self_container.resources.is_some() && other_container.resources.is_some() && self_container.resources != other_container.resources {
                return true;
            }
        }

        // Compare volumes (config maps, PVCs, etc.)
        let self_volumes = self
            .spec
            .as_ref()
            .and_then(|spec| spec.template.spec.as_ref())
            .map(|pod_spec| pod_spec.volumes.clone())
            .unwrap_or(None);

        let other_volumes = other
            .spec
            .as_ref()
            .and_then(|spec| spec.template.spec.as_ref())
            .map(|pod_spec| pod_spec.volumes.clone())
            .unwrap_or(None);

        if compare_volumes(self_volumes.as_ref(), other_volumes.as_ref()) {
            return true;
        }

        // Compare node selector. If one or both are None, there is no drift.
        // If both are NOT None, compare the values.
        if let (Some(self_node_selector), Some(other_node_selector)) = (
            self.spec
                .as_ref()
                .and_then(|spec| spec.template.spec.as_ref())
                .and_then(|pod_spec| pod_spec.node_selector.as_ref()
            ),
            other.spec
                .as_ref()
                .and_then(|spec| spec.template.spec.as_ref())
                .and_then(|pod_spec| pod_spec.node_selector.as_ref()
            ),
        ) {
            if self_node_selector != other_node_selector {
                return true;
            }
        }

        // Compare affinity
        if self.spec
            .as_ref()
            .and_then(|spec| spec.template.spec.as_ref())
            .map(|pod_spec| &pod_spec.affinity)
            != other.spec
                .as_ref()
                .and_then(|spec| spec.template.spec.as_ref())
                .map(|pod_spec| &pod_spec.affinity)
        {
            return true;
        }

        // Compare tolerations
        if self.spec
            .as_ref()
            .and_then(|spec| spec.template.spec.as_ref())
            .map(|pod_spec| &pod_spec.tolerations)
            != other.spec
                .as_ref()
                .and_then(|spec| spec.template.spec.as_ref())
                .map(|pod_spec| &pod_spec.tolerations)
        {
            return true;
        }

        // Compare pod security context. If one side is None, then no drift has occurred since it means to use the default.
        if match (
            self.spec
                .as_ref()
                .and_then(|spec| spec.template.spec.as_ref())
                .and_then(|pod_spec| pod_spec.security_context.clone()),
            other.spec
                .as_ref()
                .and_then(|spec| spec.template.spec.as_ref())
                .and_then(|pod_spec| pod_spec.security_context.clone()),
        ) {
            (Some(left), Some(right)) => left != right, // If both are Some, compare the values
            _ => false, // Every other case such as (None, Some(_)) or (Some(_), None) or (None, None) means no drift
        } {
            return true;
        }

        // Compare container security context
        let self_security_context = self.spec
            .as_ref()
            .and_then(|spec| spec.template.spec
                .as_ref()
                .map(|pod_spec| pod_spec.containers
                    .iter()
                    .map(|c| &c.security_context)
                    .collect::<Vec<_>>()
                )
            )
            .unwrap_or_default();
        let other_security_context = other.spec
            .as_ref()
            .and_then(|spec| spec.template.spec
                .as_ref()
                .map(|pod_spec| pod_spec.containers
                    .iter()
                    .map(|c| &c.security_context)
                    .collect::<Vec<_>>()
                )
            )
            .unwrap_or_default();

        if self_security_context != other_security_context {
            return true;
        }

        // Compare image pull secrets 
        if self.spec
            .as_ref()
            .and_then(|spec| spec.template.spec.as_ref())
            .map(|pod_spec| &pod_spec.image_pull_secrets)
            != other.spec
                .as_ref()
                .and_then(|spec| spec.template.spec.as_ref())
                .map(|pod_spec| &pod_spec.image_pull_secrets)
        {
            return true;
        }

        // No drift has been detected
        false
    }
}


impl FromHub<Bot> for Service {
    /// Create a Service resource from a Bot Hub
    /// 
    /// This function is responsible for creating a Service resource from a Bot Hub.
    /// 
    /// # Arguments
    /// * `bot` - The Bot CRD to create the Service resource from
    /// * `name` - The name of the Service resource
    /// * `namespace` - The namespace of the Service resource
    /// * `owner_ref` - The owner reference for the Service resource
    /// * `config` - The application configuration
    /// 
    /// # Returns
    /// The Service resource
    fn from_hub(bot: &Bot, name: &str, namespace: &str, owner_ref: OwnerReference, _config: &AppConfig) -> Self {
        let mut service = bot.spec.service.clone();
        let api = bot.spec.api.clone();

        if api.enabled {
            service.ensure_api_port(api.port);
        }

        let identifying_labels = BTreeMap::from([
            ("freqtrade.io/bot-name".to_string(), name.to_string()),
            ("app.kubernetes.io/name".to_string(), name.to_string()),
            ("app.kubernetes.io/instance".to_string(), name.to_string()),
        ]);

        Service {
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                namespace: Some(namespace.to_string()),
                owner_references: Some(vec![owner_ref]),
                annotations: service.annotations.clone(),
                labels: service.labels.clone(),
                ..Default::default()
            },
            spec: Some(ServiceSpec {
                type_: Some(service.service_type),
                selector: Some(identifying_labels),
                ports: service.ports
                    .iter()
                    .map(|port| Some(ServicePort {
                        name: Some(port.name.clone()),
                        port: port.port as i32,
                        target_port: Some(IntOrString::String(port.target_port.clone())),
                        ..Default::default()
                    }))
                    .collect(),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

impl ResourceDrift<Bot> for Service {
    /// Determine if the Service resource has drifted from another Service resource
    /// derived from the Bot CRD
    /// 
    /// # Arguments
    /// * `other` - The other Service resource to compare against
    /// 
    /// # Returns
    /// Whether the Service resource has drifted from the other Service resource
    fn has_drifted(&self, other: &Self) -> bool {
        // Compare service type
        if self.spec.as_ref().and_then(|spec| spec.type_.as_ref()) != other.spec.as_ref().and_then(|spec| spec.type_.as_ref()) {
            return true;
        }

        // Compare selector labels
        if self.spec.as_ref().and_then(|spec| spec.selector.as_ref()) != other.spec.as_ref().and_then(|spec| spec.selector.as_ref()) {
            return true;
        }

        // Compare ports
        for (self_port, other_port) in self.spec.as_ref().and_then(|spec| spec.ports.as_ref()).unwrap_or(&vec![]).iter()
            .zip(other.spec.as_ref().and_then(|spec| spec.ports.as_ref()).unwrap_or(&vec![]).iter())
            {
                if self_port.port != other_port.port
                    || self_port.name != other_port.name
                    || self_port.target_port != other_port.target_port
                    || self_port.protocol.as_deref().unwrap_or("TCP")
                        != other_port.protocol.as_deref().unwrap_or("TCP")
                {
                    return true;
                }
            }

        // No drift has been detected
        false
    }
}

pub struct BotController;

impl BotController {
    /// Create a new controller for the Bot resource
    /// 
    /// This function is responsible for creating a new controller for the bot resource.
    /// 
    /// # Arguments
    /// * `ctx` - The controller context
    /// 
    /// # Returns
    /// The controller for the Bot resource
    pub async fn create_controller<T>(ctx: Arc<Context>) -> Controller<T>
    where
        T: NamespacedCustomResource
    {
        let client = ctx.client.clone();
        let bot = Api::<T>::all(client.clone());
        
        let deployment = Api::<Deployment>::all(client.clone());
        let service = Api::<Service>::all(client.clone());
        let config_map = Api::<ConfigMap>::all(client.clone());
        let pvc = Api::<PersistentVolumeClaim>::all(client.clone());
        let secret = Api::<Secret>::all(client.clone());

        // Create the controller and watchers for the bot resource
        Controller::new(bot, watcher::Config::default())
            .owns(deployment, watcher::Config::default())
            .owns(service, watcher::Config::default())
            .owns(config_map, watcher::Config::default())
            .owns(pvc, watcher::Config::default())
            .owns(secret, watcher::Config::default())
    }

    /// Reconcile the bot resource
    /// 
    /// This function is responsible for reconciling the bot resource.
    /// 
    /// # Arguments
    /// * `bot` - The bot resource to reconcile
    /// * `ctx` - The controller context
    /// 
    /// # Returns
    /// An action to take after reconciling the bot resource
    pub async fn reconcile<T>(bot: Arc<T>, ctx: Arc<Context>) -> Result<Action>
    where
        T: NamespacedCustomResource,
        Bot: From<T>,
    {
        let client = ctx.client.clone();

        // The Bot resource is namespaced, so we need to verify that it is specified
        let namespace = match bot.namespace() {
            // If the namespace is defined, use it
            Some(namespace) => namespace,
            // If no namespace is defined, return an error since
            // we can't reconcile
            None => return Err(
                ControllerError::MissingObjectKeyError(
                    "Expected Bot to be namespaced via metadata.namespace"
                )
            )
        };
        // Get the owner reference for the bot resource to use for the other
        // resources created from the bot
        let owner_ref = bot.controller_owner_ref(&()).ok_or_else(|| {
            ControllerError::MissingObjectKeyError(
                "Expected Bot to have an owner reference"
            )
        })?;
        let api = Api::<T>::namespaced(client.clone(), &namespace);

        // Determine the action to take
        finalizer(&api, FINALIZER, bot, |event| async {
            match event {
                Finalizer::Apply(bot) => reconcile_bot(&bot, &ctx, &namespace, &owner_ref).await,
                Finalizer::Cleanup(bot) => cleanup_bot(&bot, &ctx, &namespace).await,
            }
        })
        .await
        .map_err(|e| ControllerError::FinalizerError(e.to_string()))
    }
}


/// Reconcile the bot resource
/// 
/// This function is responsible for ensuring that the bot resource is in the desired state.
/// It will create the necessary resources for the bot to run, such as a deployment, service, and
/// configuration map.
/// 
/// # Arguments
/// * `bot` - The bot resource to reconcile
/// * `ctx` - The controller context
/// * `namespace` - The namespace of the bot resource
/// * `owner_ref` - The owner reference for the bot resource
/// 
/// # Returns
/// An action to take after reconciling the bot resource
async fn reconcile_bot<T>(bot: &T, ctx: &Context, namespace: &str, owner_ref: &OwnerReference) -> Result<Action>
where
    T: NamespacedCustomResource,
    Bot: From<T>,
{
    let config_map_api = Api::<ConfigMap>::namespaced(ctx.client.clone(), namespace);
    let deployment_api = Api::<Deployment>::namespaced(ctx.client.clone(), namespace);
    let pvc_api = Api::<PersistentVolumeClaim>::namespaced(ctx.client.clone(), namespace);
    let service_api = Api::<Service>::namespaced(ctx.client.clone(), namespace);

    let hub = Bot::from(bot.clone());

    let config_map_object = ConfigMap::from_hub(
        &hub,
        bot.name_any().as_str(),
        namespace,
        owner_ref.clone(),
        &ctx.state.as_ref().unwrap().config
    );
    let deployment_object = Deployment::from_hub(
        &hub,
        bot.name_any().as_str(),
        namespace,
        owner_ref.clone(),
        &ctx.state.as_ref().unwrap().config
    );
    let service_object = Service::from_hub(
        &hub,
        bot.name_any().as_str(),
        namespace,
        owner_ref.clone(),
        &ctx.state.as_ref().unwrap().config
    );
    let pvc_object = PersistentVolumeClaim::from_hub(
        &hub,
        bot.name_any().as_str(),
        namespace,
        owner_ref.clone(),
        &ctx.state.as_ref().unwrap().config
    );

    let config_map = config_map_api.get(bot.name_any().as_str()).await.ok();
    let pvc = pvc_api.get(bot.name_any().as_str()).await.ok();
    let mut deployment = deployment_api.get(bot.name_any().as_str()).await.ok();
    let service = service_api.get(bot.name_any().as_str()).await.ok();

    let current_config_hash = deployment
        .as_ref()
        .and_then(|d| d.metadata.annotations.as_ref())
        .and_then(|annotations| annotations.get(CONFIG_HASH_ANNOTATION))
        .cloned()
        .unwrap_or_default();

    let incoming_config_hash = compute_object_hash(&config_map_object.data)
        .map_err(|e| { ControllerError::UnknownError(e.to_string())})
        .unwrap_or_default();

    if hub.status.is_none() {
        info!(
            event = "UpdatingBotStatus",
            bot = bot.name_any().as_str()
        );
        update_status(bot, ctx, namespace, &BotPhase::Pending).await?;
    }

    // If the config_map is None, OR if the config_map.data is different from the config_map_object.data,
    // apply the changes
    if config_map.is_none() || ResourceDrift::<Bot>::has_drifted(config_map.as_ref().unwrap(), &config_map_object) {
        info!(
            event = "ApplyingConfigMap",
            bot = bot.name_any().as_str()
        );
        apply(&config_map_api, config_map_object, bot.name_any().as_str()).await?;
    }

    // If the PVC is enabled, apply the PVC if it is None or different from the PVC object
    // If the PVC is not enabled, delete the PVC if it exists
    if hub.spec.pvc.enabled {
        if pvc.is_none() || ResourceDrift::<Bot>::has_drifted(pvc.as_ref().unwrap(), &pvc_object) {
            info!(
                event = "ApplyingPVC",
                bot = bot.name_any().as_str()
            );
            apply(&pvc_api, pvc_object, bot.name_any().as_str()).await?;
        }
    } else if pvc.is_some() {
        info!(
            event = "DeletingPVC",
            bot = bot.name_any().as_str()
        );
        delete(&pvc_api, bot.name_any().as_str()).await?;
    }

    // If the Deployment is None, OR if the Deployment spec is different from the Deployment object spec,
    // apply the changes
    if deployment.is_none() || ResourceDrift::<Bot>::has_drifted(deployment.as_ref().unwrap(), &deployment_object) {
        info!(
            event = "ApplyingDeployment",
            bot = bot.name_any().as_str()
        );
        deployment = Some(apply(&deployment_api, deployment_object, bot.name_any().as_str()).await?);
    }

    // If the current and incoming config hashes differ, cause a rollout for the deployment and patch the annotation
    if current_config_hash != incoming_config_hash {
        patch(&deployment_api, bot.name_any().as_str(), &Patch::Merge(json!({
            "metadata": {
                "annotations": {
                    CONFIG_HASH_ANNOTATION: incoming_config_hash,
                }
            }
        }))).await?;

        if !current_config_hash.is_empty() {
            info!(
                event = "RollingOutDeployment",
                bot = bot.name_any().as_str()
            );
            rollout(&deployment_api, bot.name_any().as_str()).await?;
        }
    }

    // If the bot status is different from the Deployment status (or None), update the bot status
    if hub.status.as_ref().is_none_or(|s| {
        BotPhase::from(
            deployment
                .as_ref()
                .unwrap()
                .status
                .clone()
                .unwrap()
            )
            .to_string() != s.phase
    }) {
        info!(
            event = "UpdatingBotStatus",
            bot = bot.name_any().as_str(),
            status = BotPhase::from(
                deployment
                    .as_ref()
                    .unwrap()
                    .status
                    .clone()
                    .unwrap()
            )
            .to_string()
        );
        update_status(
            bot,
            ctx,
            namespace,
            &BotPhase::from(
                deployment
                    .as_ref()
                    .unwrap()
                    .status
                    .clone()
                    .unwrap()
            )
        ).await?;
    }

    // If the API is enabled, apply the Service if it is None or different from the Service object
    // If the API is not enabled, delete the Service if it exists
    if hub.spec.api.enabled {
        if service.is_none() || ResourceDrift::<Bot>::has_drifted(service.as_ref().unwrap(), &service_object) {
            info!(
                event = "ApplyingService",
                bot = bot.name_any().as_str()
            );
            apply(&service_api, service_object, bot.name_any().as_str()).await?;
        }
    } else if service.is_some() {
        info!(
            event = "DeletingService",
            bot = bot.name_any().as_str()
        );
        delete(&service_api, bot.name_any().as_str()).await?;
    }

    Ok(Action::requeue(Duration::from_secs(30)))
}

/// Cleanup the bot resource
/// 
/// This function is responsible for cleaning up the bot resource.
/// 
/// # Arguments
/// * `bot` - The bot resource to cleanup
/// * `ctx` - The controller context
/// * `namespace` - The namespace of the bot resource
/// 
/// # Returns
/// An action to take after cleaning up the bot resource
async fn cleanup_bot<T>(bot: &T, ctx: &Context, namespace: &str) -> Result<Action>
where
    T: NamespacedCustomResource,
    Bot: From<T>,
{
    update_status(bot, ctx, namespace, &BotPhase::Deleting).await?;

    Ok(Action::await_change())
}

/// Update the status of the bot resource
/// 
/// This function is responsible for updating the status of the bot resource.
/// 
/// # Arguments
/// * `bot` - The bot resource to update
/// * `ctx` - The controller context
/// * `namespace` - The namespace of the bot resource
/// * `phase` - The phase to set the bot resource to
///
/// # Returns
/// A result indicating success or failure
async fn update_status<T>(bot: &T, ctx: &Context, namespace: &str, phase: &BotPhase) -> Result<()>
where
    T: NamespacedCustomResource,
    Bot: From<T>,
{
    let client = ctx.client.clone();
    let api = Api::<T>::namespaced(client.clone(), namespace);

    api.patch_status(
        &bot.name_any(),
        &PatchParams::apply(FIELD_MANAGER),
        &Patch::Merge(json!({
            "status": BotStatus {
                phase: phase.to_string(),
                last_updated: Some(Utc::now()),
            }
        })),
    ).await?;

    Ok(())
}

/// Create an environment variable from a secret item
/// 
/// This function is responsible for creating an environment variable from a secret item.
/// 
/// # Arguments
/// * `name` - The name of the environment variable
/// * `secret_item` - The secret item to create the environment variable from
fn create_secret_env_var(name: &str, secret_item: &Option<SecretItem>) -> EnvVar {
    EnvVar {
        name: name.to_string(),
        value: match secret_item {
            Some(SecretItem::Value { value }) => Some(value.clone()),
            _ => None,
        },
        value_from: match secret_item {
            Some(SecretItem::SecretKeyRef { secret_key_ref }) => Some(EnvVarSource {
                secret_key_ref: Some(SecretKeySelector {
                    name: secret_key_ref.name.clone(),
                    key: secret_key_ref.key.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            _ => None,
        }
    }
}

/// Create an environment variable
/// 
/// This function is responsible for creating an environment variable.
/// 
/// # Arguments
/// * `name` - The name of the environment variable
/// * `value` - The value of the environment variable
/// 
/// # Returns
/// The environment variable
fn create_env_var(name: &str, value: Option<String>) -> EnvVar {
    EnvVar {
        name: name.to_string(),
        value: value.map(|value| value.to_string()),
        ..Default::default()
    }
}


/// Compare container ports
/// 
/// This function is responsible for comparing the ports of two container ports.
/// 
/// # Arguments
/// * `self_ports` - The ports of the first container
/// * `other_ports` - The ports of the second container
/// 
/// # Returns
/// Whether the ports are different
fn compare_container_ports(self_ports: Option<&Vec<ContainerPort>>, other_ports: Option<&Vec<ContainerPort>>) -> bool {
    match (self_ports, other_ports) {
        (Some(self_ports), Some(other_ports)) => {
            if self_ports.len() != other_ports.len() {
                return true;
            }

            for (self_port, other_port) in self_ports.iter().zip(other_ports.iter()) {
                if self_port.container_port != other_port.container_port
                    || self_port.name != other_port.name
                    || self_port.protocol.as_deref().unwrap_or("TCP")
                        != other_port.protocol.as_deref().unwrap_or("TCP")
                {
                    return true;
                }
            }

            false
        },
        (None, None) => false,
        _ => true,
    }
}


/// Compare environment variables
/// 
/// This function is responsible for comparing environment variables.
/// 
/// # Arguments
/// * `self_vars` - The environment variables of the first container
/// * `other_vars` - The environment variables of the second container
/// 
/// # Returns
/// Whether the environment variables are different
fn compare_env_vars(self_vars: Option<&Vec<EnvVar>>, other_vars: Option<&Vec<EnvVar>>) -> bool {
    match (self_vars, other_vars) {
        (Some(self_vars), Some(other_vars)) => {
            if self_vars.len() != other_vars.len() {
                return true;
            }

            let mut self_vars = self_vars.clone();
            let mut other_vars = other_vars.clone();

            // Sort the environment variables by name (or any other field you want to compare)
            self_vars.sort_by(|a, b| a.name.cmp(&b.name));
            other_vars.sort_by(|a, b| a.name.cmp(&b.name));

            if self_vars != other_vars {
                return true;
            }

            false
        },
        (None, None) => false,
        _ => true,
    }
}

/// Compare volumes
/// 
/// This function is responsible for comparing volumes.
/// 
/// # Arguments
/// * `self_vols` - The volumes of the first container
/// * `other_vols` - The volumes of the second container
/// 
/// # Returns
/// Whether the volumes are different
fn compare_volumes(self_vols: Option<&Vec<Volume>>, other_vols: Option<&Vec<Volume>>) -> bool {
    match (self_vols, other_vols) {
        (Some(self_vols), Some(other_vols)) => {
            // If lengths are different, they are not equal
            if self_vols.len() != other_vols.len() {
                return true;
            }

            // Sort by name to ensure comparison is order-independent
            let mut self_sorted = self_vols.clone();
            let mut other_sorted = other_vols.clone();
            self_sorted.sort_by(|a, b| a.name.cmp(&b.name));
            other_sorted.sort_by(|a, b| a.name.cmp(&b.name));

            // Compare each volume
            for (self_vol, other_vol) in self_sorted.iter().zip(other_sorted.iter()) {
                if !volumes_are_equal(self_vol, other_vol) {
                    return true;
                }
            }

            false // They are equal
        },
        (None, None) => false, // Both are None, considered equal
        _ => true, // One is Some, the other is None, not equal
    }
}

/// Function to compare two Volume objects, handling special cases of None and default values
///
/// # Arguments
/// * `self_vol` - The first Volume object to compare
/// * `other_vol` - The second Volume object to compare
/// 
/// # Returns
/// Whether the volumes are equal
fn volumes_are_equal(self_vol: &Volume, other_vol: &Volume) -> bool {
    // Start by comparing volumes using default `PartialEq` for all fields except config_map
    if self_vol == other_vol {
        return true;
    }

    // Now handle the case where default_mode should be considered equivalent
    match (&self_vol.config_map, &other_vol.config_map) {
        (Some(self_config), Some(other_config)) => {
            // Compare all fields except default_mode
            let mut are_equal = self_config.name == other_config.name
                && self_config.items == other_config.items
                && self_config.optional == other_config.optional;

            // Handle special case where None and Some(420) for default_mode are considered equal
            are_equal &= match (self_config.default_mode, other_config.default_mode) {
                (None, Some(420)) | (Some(420), None) => true,
                (self_mode, other_mode) => self_mode == other_mode,
            };

            are_equal
        },
        // If both config_map fields are None, they are considered equal
        (None, None) => true,
        _ => false, // One is Some, the other is None, not equal
    }
}
