use std::time::Duration;

use grafbase_sdk::{
    AuthenticationExtension, Error, GatewayHeaders,
    host_io::cache::{self, CachedItem},
    types::{Configuration, ErrorResponse, StatusCode, Token},
};

#[derive(AuthenticationExtension)]
struct CachingProvider {
    config: ProviderConfig,
}

#[derive(Debug, serde::Deserialize)]
struct ProviderConfig {
    cache_config: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Jwks {
    key: String,
}

impl AuthenticationExtension for CachingProvider {
    fn new(config: Configuration) -> Result<Self, Error> {
        let config: ProviderConfig = config.deserialize()?;

        Ok(Self { config })
    }

    fn authenticate(&mut self, headers: &GatewayHeaders) -> Result<Token, ErrorResponse> {
        let header = headers
            .get("Authorization")
            .ok_or_else(|| ErrorResponse::new(StatusCode::UNAUTHORIZED))?
            .to_str()
            .map(str::to_string)
            .unwrap_or_default();

        let value = headers
            .get("value")
            .and_then(|v| v.to_str().ok().map(str::to_string))
            .unwrap_or_else(|| String::from("default"));

        let cache_key = format!("auth:{}:{header}", self.config.cache_config);

        let jwks: Jwks = cache::get(&cache_key, || {
            std::thread::sleep(Duration::from_millis(300));

            let jwks = Jwks { key: value };
            let item = CachedItem::new(jwks, None);

            Ok(item)
        })
        .unwrap();

        Ok(Token::from_bytes(serde_json::to_vec(&jwks).unwrap()))
    }
}
