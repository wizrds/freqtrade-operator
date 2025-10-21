use k8s_openapi::apimachinery::pkg::apis::meta::v1::OwnerReference;

use ft_operator_common::config::AppConfig;

use crate::crd::hub::traits::Hub;

// Trait to get a Resource from a Hub CustomResourceDefinition
pub trait FromHub<T>
where
    T: Hub,
{
    fn from_hub(hub: &T, name: &str, namespace: &str, owner_ref: OwnerReference, config: &AppConfig) -> Self;
}

// Trait to detect if a child Resource has drifted from another instance
// of the same Resource, where Resource has implemented the FromHub trait
pub trait ResourceDrift<T>
where
    T: Hub,
{
    fn has_drifted(&self, other: &Self) -> bool;
}