use std::time::Duration;

use grafbase_sdk::{
    AuthenticationExtension, Authenticator, Extension, Headers,
    host_io::cache::{self, CachedItem},
    types::{Configuration, ErrorResponse, SchemaDirective, StatusCode, Token},
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

impl Extension for CachingProvider {
    fn new(_: Vec<SchemaDirective>, config: Configuration) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        let config: ProviderConfig = config.deserialize()?;

        Ok(Self { config })
    }
}

impl Authenticator for CachingProvider {
    fn authenticate(&mut self, headers: Headers) -> Result<Token, ErrorResponse> {
        let header = headers
            .get("Authorization")
            .ok_or_else(|| ErrorResponse::new(StatusCode::UNAUTHORIZED))?;

        let value = headers.get("value").unwrap_or_else(|| String::from("default"));

        let cache_key = format!("auth:{}:{header}", self.config.cache_config);

        let jwks: Jwks = cache::get(&cache_key, || {
            std::thread::sleep(Duration::from_millis(300));

            let jwks = Jwks { key: value };
            let item = CachedItem::new(jwks, None);

            Ok(item)
        })
        .unwrap();

        Ok(Token::new(jwks))
    }
}
