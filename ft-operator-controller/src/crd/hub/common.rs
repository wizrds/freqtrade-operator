use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

use crate::crd::v1alpha1;


#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(untagged, rename_all = "camelCase")]
pub enum SecretItem {
    Value { value: String },
    #[serde(rename_all = "camelCase")]
    SecretKeyRef { secret_key_ref: SecretKeyRef },
}

impl From<v1alpha1::common::SecretItem> for SecretItem {
    fn from(secret_item: v1alpha1::common::SecretItem) -> Self {
        match secret_item {
            v1alpha1::common::SecretItem::Value { value } => SecretItem::Value { value },
            v1alpha1::common::SecretItem::SecretKeyRef { secret_key_ref } => SecretItem::SecretKeyRef { secret_key_ref: secret_key_ref.into() },
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct SecretKeyRef {
    pub name: String,
    pub key: String,
}

impl From<v1alpha1::common::SecretKeyRef> for SecretKeyRef {
    fn from(secret_key_ref: v1alpha1::common::SecretKeyRef) -> Self {
        SecretKeyRef {
            name: secret_key_ref.name,
            key: secret_key_ref.key,
        }
    }
}