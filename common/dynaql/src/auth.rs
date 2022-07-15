#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Auth {
    pub oidc_providers: Vec<OidcAuthProvider>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct OidcAuthProvider {
    pub issuer: url::Url,
}

impl Auth {
    pub fn oidc_provider(&self) -> Option<&OidcAuthProvider> {
        // TODO: support multiple OIDC providers (?)
        self.oidc_providers.first()
    }
}
