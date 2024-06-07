pub static X_GRAFBASE_CLIENT_NAME: http::HeaderName = http::HeaderName::from_static("x-grafbase-client-name");
pub static X_GRAFBASE_CLIENT_VERSION: http::HeaderName = http::HeaderName::from_static("x-grafbase-client-version");

#[derive(Debug, Clone)]
pub struct Client {
    pub name: String,
    pub version: Option<String>,
}

impl Client {
    pub fn maybe_new(name: Option<String>, version: Option<String>) -> Option<Self> {
        name.map(|name| Self { name, version })
    }

    pub fn extract_from(headers: &http::HeaderMap) -> Option<Self> {
        let name = headers.get(&X_GRAFBASE_CLIENT_NAME).and_then(|v| v.to_str().ok())?;
        let version = headers
            .get(&X_GRAFBASE_CLIENT_VERSION)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        Some(Client {
            name: name.to_string(),
            version,
        })
    }
}
