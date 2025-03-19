use grafbase_sdk::{
    AuthenticationExtension, Error, Headers,
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

    fn authenticate(&mut self, headers: Headers) -> Result<Token, ErrorResponse> {
        let header = headers.get("Authorization").ok_or_else(|| {
            ErrorResponse::new(StatusCode::UNAUTHORIZED)
                .with_error(Error::new("Not passing through on my watch! SDK-09"))
        })?;

        let value = headers.get("value").unwrap_or_else(|| String::from("default"));

        let cache_key = format!("auth:{}:{header}", self.config.cache_config);

        let jwks: Jwks = cache::get(&cache_key, || {
            let jwks = Jwks { key: value };
            let item = CachedItem::new(jwks, None);

            Ok(item)
        })
        .unwrap();

        Ok(Token::from_bytes(serde_json::to_vec(&jwks).unwrap()))
    }
}
