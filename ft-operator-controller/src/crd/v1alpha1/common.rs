use serde::{Deserialize, Serialize};
use schemars::JsonSchema;


#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(untagged, rename_all = "camelCase")]
pub enum SecretItem {
    /// The value of the secret inline
    Value { value: String },
    #[serde(rename_all = "camelCase")]
    /// A reference to a Secret in the same namespace with the value
    SecretKeyRef { secret_key_ref: SecretKeyRef },
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct SecretKeyRef {
    /// The name of the Secret to reference
    pub name: String,
    /// The key in the Secret to reference
    pub key: String,
}