use std::collections::{BTreeSet, HashMap};

use common_types::auth::Operations;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthConfig {
    pub allowed_private_ops: Operations,

    #[serde(default)]
    pub allowed_public_ops: Operations,

    #[serde(with = "::serde_with::rust::maps_duplicate_key_is_error")]
    pub allowed_group_ops: HashMap<String, Operations>,

    pub allowed_owner_ops: Operations,

    pub provider: Option<AuthProvider>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthProvider {
    Oidc(OidcProvider),
    Jwks(JwksProvider),
    Jwt(JwtProvider),
    Authorizer(AuthorizerProvider),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OidcProvider {
    pub issuer: String,            // For verifying the "iss" claim.
    pub issuer_base_url: url::Url, // For deriving the OIDC discovery URL.
    pub groups_claim: String,      // Name of the claim containing the groups the subject belongs to.
    pub client_id: Option<String>, // Used for verifying that the supplied value is in the "aud" claim.
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JwksProvider {
    pub jwks_endpoint: url::Url, // URL of the JWKS endpoint. E.g. https://example.com/.well-known/jwks.json
    pub issuer: Option<String>,  // Used for verifying the "iss" claim.
    pub groups_claim: String,    // Name of the claim containing the groups the subject belongs to.
    pub client_id: Option<String>, // Used for verifying that the supplied value is in the "aud" claim.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtProvider {
    pub issuer: String,

    pub groups_claim: String,

    pub client_id: Option<String>,

    #[serde(serialize_with = "serialize_secret_string")]
    pub secret: SecretString,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizerProvider {
    pub name: String,
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

            allowed_public_ops: Operations::empty(),

            allowed_group_ops: HashMap::new(),

            allowed_owner_ops: Operations::empty(),

            provider: None,
        }
    }
}

impl AuthConfig {
    pub fn private_public_and_group_based_ops(&self, groups_from_token: &BTreeSet<String>) -> Operations {
        // Add ops for each group contained in ID token
        // Minimum ops are that of any signed-in user union public ops.
        let minimum_ops = self.allowed_public_ops.union(self.allowed_private_ops);
        let groups = self.allowed_group_ops.clone().into_keys().collect();
        groups_from_token
            .intersection(&groups)
            .fold(minimum_ops, |ops, group| ops.union(self.allowed_group_ops[group]))
    }

    pub fn owner_based_ops(&self) -> Operations {
        self.allowed_owner_ops
    }
}
