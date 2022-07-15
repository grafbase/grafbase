#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Auth {
    pub oidc_providers: Vec<OidcProvider>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct OidcProvider {
    pub issuer: url::Url,
}
