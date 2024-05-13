use std::sync::OnceLock;

pub const CREDENTIALS_FILE: &str = "credentials.json";
pub const AUTH_URL: &str = "https://app.grafbase.com/auth/cli";
pub const PACKAGE_JSON: &str = "package.json";
pub const TAR_CONTENT_TYPE: &str = "application/x-tar";
pub const GRAFBASE_ACCESS_TOKEN_ENV_VAR: &str = "GRAFBASE_ACCESS_TOKEN";

const API_URL: &str = "https://api.grafbase.com/graphql";
const DASHBOARD_URL: &str = "https://app.grafbase.com";

pub fn api_url() -> &'static str {
    static API_URL: OnceLock<String> = OnceLock::new();

    API_URL.get_or_init(|| match std::env::var("GRAFBASE_API_URL").ok() {
        Some(url) => url,
        None => self::API_URL.to_string(),
    })
}

pub fn dashboard_url() -> &'static str {
    static DASHBOARD_URL: OnceLock<String> = OnceLock::new();

    DASHBOARD_URL.get_or_init(|| match std::env::var("GRAFBASE_DASHBOARD_URL").ok() {
        Some(url) => url,
        None => self::DASHBOARD_URL.to_string(),
    })
}
