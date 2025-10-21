use kube::{
    api::{Api, Patch, PatchParams}, core::response::Status, runtime::controller::Action, Client, Resource
};
use k8s_openapi::api::apps::v1::Deployment;
use std::sync::Arc;
use std::fmt::Debug;
use tokio::time::Duration;
use either::Either;
use serde::Serialize;
use serde::de::DeserializeOwned;

use ft_operator_common::telemetry::error;

use crate::controller::context::Context;
use crate::error::{ControllerError, Result};

pub static FIELD_MANAGER: &str = "operator.freqtrade.io";


/// Create a new kube client by inferring the kubeconfig from the environment
/// or the default service account
///
/// # Returns
/// A Result containing the kube Client or an error
pub async fn create_k8s_client() -> Result<Client> {
    Client::try_default().await.map_err(ControllerError::from)
}

/// Error policy to log the error and requeue the object after 30 seconds
/// 
/// # Arguments
/// * `_object`: The object that caused the error
/// * `_error`: The error that occurred
/// * `_ctx`: The context of the controller
///
/// # Returns
/// An Action to requeue the object after 30 seconds
pub fn error_policy<T>(_object: Arc<T>, _error: &ControllerError, _ctx: Arc<Context>) -> Action {
    error!(
        event = "Error",
        error = %_error,
    );
    Action::requeue(Duration::from_secs(30))
}

/// Apply a Resource to the cluster
/// 
/// # Arguments
/// * `api`: The API client for the resource type
/// * `obj`: The object to apply
/// * `name`: The name of the object
/// 
/// # Returns
/// A Result containing the applied object or an error
pub async fn apply<T>(api: &Api<T>, obj: T, name: &str) -> Result<T>
where
    T: Clone + Debug + Serialize + DeserializeOwned + Resource<DynamicType = ()>,
{
    api.patch(
        name,
        &PatchParams::apply(FIELD_MANAGER),
        &Patch::Apply(obj),
    ).await.map_err(ControllerError::from)
}

/// Delete a Resource
/// 
/// # Arguments
/// * `api`: The API client for the resource type
/// * `name`: The name of the object to delete
/// 
/// # Returns
/// A Result containing either the deleted object or a Status indicating the deletion was successful
pub async fn delete<T>(api: &Api<T>, name: &str) -> Result<Either<T, Status>>
where
    T: Clone + Debug + Serialize + DeserializeOwned + Resource<DynamicType = ()>,
{
    api.delete(
        name,
        &Default::default()
    ).await.map_err(ControllerError::from)
}

/// Patch a Resource
/// 
/// # Arguments
/// * `api`: The API client for the resource type
/// * `name`: The name of the object to patch
/// * `patch`: The patch to apply
/// 
/// # Returns
/// A Result containing the patched object or an error
pub async fn patch<T>(api: &Api<T>, name: &str, patch: &Patch<serde_json::Value>) -> Result<T>
where
    T: Clone + Debug + Serialize + DeserializeOwned + Resource<DynamicType = ()>,
{
    api.patch(name, &PatchParams::apply(FIELD_MANAGER), patch).await.map_err(ControllerError::from)
}

/// Rollout a Deployment
/// 
/// # Arguments
/// * `api`: The API client for the Deployment resource
/// * `name`: The name of the Deployment to rollout
/// 
/// # Returns
/// A Result indicating success or an error
pub async fn rollout(api: &Api<Deployment>, name: &str) -> Result<()> {
    patch::<Deployment>(api, name, &Patch::Merge(
        serde_json::json!({
            "spec": {
                "template": {
                    "metadata": {
                        "annotations": {
                            "kube.kubernetes.io/restartedAt": chrono::Utc::now().to_rfc3339()
                        }
                    }
                }
            }
        }),
    )).await?;

    Ok(())
}