use std::collections::HashSet;

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Auth {
    pub allow_anonymous_access: bool,

    pub allow_private_access: bool,

    #[serde(with = "::serde_with::rust::sets_duplicate_value_is_error")]
    pub allowed_groups: HashSet<String>,

    pub oidc_providers: Vec<OidcProvider>,
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OidcProvider {
    pub issuer: url::Url,
}
