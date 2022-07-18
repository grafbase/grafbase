#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Auth {
    pub oidc_providers: Vec<OidcProvider>,
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OidcProvider {
    pub issuer: url::Url,
    pub groups: Option<Vec<String>>,
    pub groups_claim: String,
}
