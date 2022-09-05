use std::collections::HashSet;
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_with::rust::sets_duplicate_value_is_error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthConfig {
    pub allowed_anonymous_ops: Operations,

    pub allowed_private_ops: Operations,

    #[serde(with = "sets_duplicate_value_is_error")]
    pub allowed_groups: HashSet<String>,
    pub allowed_group_ops: Operations,

    pub oidc_providers: Vec<OidcProvider>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OidcProvider {
    pub issuer: url::Url,
}

impl Default for AuthConfig {
    fn default() -> Self {
        AuthConfig {
            allowed_anonymous_ops: Operations::all(),

            allowed_private_ops: Operations::empty(),

            allowed_groups: HashSet::new(),
            allowed_group_ops: Operations::empty(),

            oidc_providers: vec![],
        }
    }
}

bitflags::bitflags! {
    #[allow(clippy::unsafe_derive_deserialize)]
    #[derive(Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct Operations: u8 {
        const CREATE = 1 << 0;
        const GET    = 1 << 1; // More granual read access
        const LIST   = 1 << 2; // More granual read access
        const UPDATE = 1 << 3;
        const DELETE = 1 << 4;
        const READ   = Self::GET.bits | Self::LIST.bits;
    }
}

impl fmt::Display for Operations {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{self:?}").to_lowercase())
    }
}
