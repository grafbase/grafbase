use std::collections::HashSet;

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Auth {
    pub oidc_providers: Vec<OidcProvider>,
    pub allowed_groups: HashSet<String>,
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OidcProvider {
    pub issuer: url::Url,
}
