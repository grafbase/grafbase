use std::collections::HashSet;

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Auth {
    pub oidc_providers: Vec<OidcProvider>,

    #[serde(with = "::serde_with::rust::sets_duplicate_value_is_error")]
    pub allowed_groups: HashSet<String>,
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OidcProvider {
    pub issuer: url::Url,
}
