pub mod hub;
pub mod utils;
pub mod v1alpha1;

use kube::{Resource, CustomResourceExt, core::object::{HasStatus, HasSpec}};
use k8s_openapi::{NamespaceResourceScope, ClusterResourceScope};
use schemars::JsonSchema;
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;

pub trait NamespacedCustomResource:
    Clone
        + Resource<Scope = NamespaceResourceScope, DynamicType = ()>
        + CustomResourceExt
        + HasStatus
        + HasSpec
        + JsonSchema
        + DeserializeOwned
        + Serialize
        + Debug
        + Send
        + Sync
        + 'static
{}

impl<T> NamespacedCustomResource for T
where
    T: Clone
        + Resource<Scope = NamespaceResourceScope, DynamicType = ()>
        + CustomResourceExt
        + HasStatus
        + HasSpec
        + JsonSchema
        + DeserializeOwned
        + Serialize
        + Debug
        + Send
        + Sync
        + 'static
{}

pub trait ClusterCustomResource:
    Clone
        + Resource<Scope = ClusterResourceScope, DynamicType = ()>
        + CustomResourceExt
        + HasStatus
        + HasSpec
        + JsonSchema
        + DeserializeOwned
        + Serialize
        + Debug
        + Send
        + Sync
        + 'static
{}

impl<T> ClusterCustomResource for T
where
    T: Clone
        + Resource<Scope = ClusterResourceScope, DynamicType = ()>
        + CustomResourceExt
        + HasStatus
        + HasSpec
        + JsonSchema
        + DeserializeOwned
        + Serialize
        + Debug
        + Send
        + Sync
        + 'static
{}