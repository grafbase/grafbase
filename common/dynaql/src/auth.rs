use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use serde_with::rust::sets_duplicate_value_is_error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthConfig {
    pub allowed_anonymous_ops: Operations_,

    pub allowed_private_ops: Operations_,

    #[serde(with = "sets_duplicate_value_is_error")]
    pub allowed_groups: HashSet<String>,
    pub allowed_group_ops: Operations_,

    pub oidc_providers: Vec<OidcProvider>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OidcProvider {
    pub issuer: url::Url,
}

impl Default for AuthConfig {
    fn default() -> Self {
        AuthConfig {
            allowed_anonymous_ops: Operations_::all(),

            allowed_private_ops: Operations_::empty(),

            allowed_groups: HashSet::new(),
            allowed_group_ops: Operations_::empty(),

            oidc_providers: vec![],
        }
    }
}

bitflags::bitflags! {
    #[allow(clippy::unsafe_derive_deserialize)]
    #[derive(Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct Operations_: u8 {
        const CREATE = 0b0000_0001;
        const GET    = 0b0000_0010; // More granual read access
        const LIST   = 0b0000_0100; // More granual read access
        const UPDATE = 0b0000_1000;
        const DELETE = 0b0001_0000;
        const READ   = Self::GET.bits | Self::LIST.bits;
    }
}
