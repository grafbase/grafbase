use grafbase::auth::Operations;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthConfig {
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

    pub client_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtProvider {
    pub issuer: String,

    pub groups_claim: String,

    pub client_id: Option<String>,

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
            allowed_private_ops: Operations::empty(),

            allowed_group_ops: HashMap::new(),

            allowed_owner_ops: Operations::empty(),

            oidc_providers: vec![],

            jwt_providers: vec![],
        }
    }
}

impl AuthConfig {
    pub fn private_and_group_based_ops(&self, groups_from_token: &HashSet<String>) -> Operations {
        // Add ops for each group contained in ID token
        // Minimum ops are that of any signed-in user, if present
        let groups = self.allowed_group_ops.clone().into_keys().collect();
        groups_from_token
            .intersection(&groups)
            .fold(self.allowed_private_ops, |ops, group| {
                ops.union(self.allowed_group_ops[group])
            })
    }

    pub fn owner_based_ops(&self) -> Operations {
        self.allowed_owner_ops
    }

    pub fn allowed_ops(&self, groups_from_token: Option<&HashSet<String>>) -> Operations {
        match groups_from_token {
            Some(groups) => self.private_and_group_based_ops(groups),
            None => grafbase::auth::API_KEY_OPS,
        }
    }
}
