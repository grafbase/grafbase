#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CsrfConfig {
    #[serde(default)]
    enabled: bool,
}

impl CsrfConfig {
    /// If true, we expect a header `X-Grafbase-CSRF` to be
    /// set in every request
    pub fn enabled(&self) -> bool {
        self.enabled
    }
}
