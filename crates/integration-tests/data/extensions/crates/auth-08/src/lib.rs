use grafbase_sdk::{
    AuthenticationExtension, Authenticator, Error, Extension, Headers,
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

impl Extension for CachingProvider {
    fn new(
        _: Vec<grafbase_sdk::types::SchemaDirective>,
        config: Configuration,
    ) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        let config: ProviderConfig = config.deserialize()?;

        Ok(Self { config })
    }
}

impl Authenticator for CachingProvider {
    fn authenticate(&mut self, headers: Headers) -> Result<Token, ErrorResponse> {
        let header = headers.get("Authorization").ok_or_else(|| {
            ErrorResponse::new(StatusCode::UNAUTHORIZED)
                .with_error(Error::new("Not passing through on my watch! SDK-08"))
        })?;

        let value = headers.get("value").unwrap_or_else(|| String::from("default"));
        let cache_key = format!("auth:{}:{header}", self.config.cache_config);

        let jwks: Jwks = cache::get(&cache_key, || {
            let jwks = Jwks { key: value };
            let item = CachedItem::new(jwks, None);

            Ok(item)
        })
        .unwrap();

        Ok(Token::new(jwks))
    }
}
