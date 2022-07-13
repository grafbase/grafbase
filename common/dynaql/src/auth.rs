pub const OIDC_PROVIDER: &str = "oidc";

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Auth {
    pub providers: Vec<AuthProvider>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AuthProvider {
    pub r#type: String, // TODO: turn this into an enum once we support more providers
    pub issuer: url::Url,
}

impl Auth {
    pub fn oidc_provider(&self) -> Option<&AuthProvider> {
        self.providers.iter().find(|p| p.r#type == OIDC_PROVIDER)
    }
}
