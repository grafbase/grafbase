use std::collections::{HashMap, HashSet};
use std::fmt;

use secrecy::SecretString;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthConfig {
    pub allowed_anonymous_ops: Operations,

    pub allowed_private_ops: Operations,

    #[serde(with = "::serde_with::rust::maps_duplicate_key_is_error")]
    pub allowed_group_ops: HashMap<String, Operations>,

    pub allowed_owner_ops: Operations,

    pub oidc_providers: Vec<OidcProvider>,

    pub jwt_providers: Vec<JwtProvider>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OidcProvider {
    pub issuer: url::Url,

    pub groups_claim: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtProvider {
    pub issuer: url::Url,

    pub groups_claim: String,

    #[serde(serialize_with = "serialize_secret_string")]
    pub secret: SecretString,
}

fn serialize_secret_string<S>(secret: &SecretString, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use secrecy::ExposeSecret;
    serializer.serialize_str(secret.expose_secret())
}

impl PartialEq for JwtProvider {
    fn eq(&self, other: &Self) -> bool {
        use secrecy::ExposeSecret;
        self.issuer == other.issuer
            && self.groups_claim == other.groups_claim
            && self.secret.expose_secret() == other.secret.expose_secret()
    }
}

impl Eq for JwtProvider {}

impl Default for AuthConfig {
    fn default() -> Self {
        AuthConfig {
            allowed_anonymous_ops: Operations::all(),

            allowed_private_ops: Operations::empty(),

            allowed_group_ops: HashMap::new(),

            allowed_owner_ops: Operations::empty(),

            oidc_providers: vec![],

            jwt_providers: vec![],
        }
    }
}

impl AuthConfig {
    pub fn api_key_ops(&self) -> Operations {
        self.allowed_anonymous_ops
    }

    pub fn token_ops(&self, groups_from_token: &HashSet<String>) -> Operations {
        // Add ops for each group contained in ID token
        // Minimum ops are that of any signed-in user, if present
        let groups = self.allowed_group_ops.clone().into_keys().collect();
        groups_from_token
            .intersection(&groups)
            .fold(self.allowed_private_ops, |ops, group| {
                ops.union(self.allowed_group_ops[group])
            })
    }

    pub fn allowed_ops(&self, groups_from_token: Option<&HashSet<String>>) -> Operations {
        match groups_from_token {
            Some(groups) => self.token_ops(groups),
            None => self.api_key_ops(),
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
