use std::time::Duration;

use grafbase_sdk::{
    AuthenticationExtension,
    host_io::cache::Cache,
    types::{Configuration, Error, ErrorResponse, Headers, RequestContext, Token},
};

#[derive(AuthenticationExtension)]
struct CachingProvider {
    config: ProviderConfig,
    cache: Cache,
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

        Ok(Self {
            config,
            cache: Cache::builder("test", 128).build(),
        })
    }

    fn authenticate(&mut self, _ctx: &RequestContext, headers: &Headers) -> Result<Token, ErrorResponse> {
        let header = headers
            .get("Authorization")
            .ok_or_else(ErrorResponse::unauthorized)?
            .to_str()
            .map(str::to_string)
            .unwrap_or_default();

        let value = headers
            .get("value")
            .and_then(|v| v.to_str().ok().map(str::to_string))
            .unwrap_or_else(|| String::from("default"));

        let cache_key = format!("auth:{}:{header}", self.config.cache_config);

        let jwks_bytes = self
            .cache
            .get_or_insert(&cache_key, || {
                std::thread::sleep(Duration::from_millis(300));

                let jwks = Jwks { key: value };

                serde_json::to_vec(&jwks).map(|bytes| ((), bytes))
            })
            .unwrap()
            .1;

        Ok(Token::from_bytes(jwks_bytes))
    }
}
