use std::time::Duration;

use grafbase_sdk::{
    types::{Cache, CachedItem, Configuration, Directive, ErrorResponse, StatusCode, Token},
    AuthenticationExtension, Authenticator, Extension, Headers,
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
    fn new(_: Vec<Directive>, config: Configuration) -> Result<Self, Box<dyn std::error::Error>>
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

        let cache_key = format!("auth:{}:{header}", self.config.cache_config);

        let jwks: Jwks = Cache::get(&cache_key, || {
            std::thread::sleep(Duration::from_millis(300));

            let jwks = Jwks { key: header };

            let item = CachedItem::new(jwks, Some(Duration::from_millis(900)));

            Ok(item)
        })
        .unwrap();

        Ok(Token::new(jwks))
    }
}
