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
#[serde(default)]
struct ProviderConfig {
    header_name: String,
    cache_key_prefix: String,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            header_name: "Authorization".to_owned(),
            cache_key_prefix: "test".to_owned(),
        }
    }
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
        let config: Option<ProviderConfig> = config.deserialize()?;

        Ok(Self {
            config: config.unwrap_or_default(),
        })
    }
}

impl Authenticator for CachingProvider {
    fn authenticate(&mut self, headers: Headers) -> Result<Token, ErrorResponse> {
        let auth = headers.get(&self.config.header_name).ok_or_else(|| {
            ErrorResponse::new(StatusCode::UNAUTHORIZED)
                .with_error(Error::new("Not passing through on my watch! SDK-08"))
        })?;

        let value = headers.get("value").unwrap_or_else(|| String::from("default"));

        let cache_key = format!("{}:{auth}", self.config.cache_key_prefix);

        let jwks: Jwks = cache::get(&cache_key, || {
            let jwks = Jwks { key: value };
            let item = CachedItem::new(jwks, None);

            Ok(item)
        })
        .unwrap();

        Ok(Token::new(format!("sdk08:{auth}:{}", jwks.key)))
    }
}
