use grafbase_sdk::{
    AuthenticationExtension,
    host_io::{
        cache::{self, CachedItem},
        http::StatusCode,
    },
    types::{Configuration, Error, ErrorResponse, GatewayHeaders, Token},
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

impl AuthenticationExtension for CachingProvider {
    fn new(config: Configuration) -> Result<Self, Error> {
        let config: Option<ProviderConfig> = config.deserialize()?;

        Ok(Self {
            config: config.unwrap_or_default(),
        })
    }

    fn authenticate(&mut self, headers: &GatewayHeaders) -> Result<Token, ErrorResponse> {
        let auth = headers.get(&self.config.header_name).ok_or_else(|| {
            ErrorResponse::new(StatusCode::UNAUTHORIZED)
                .with_error(Error::new("Not passing through on my watch! SDK-15"))
        })?;

        let value = headers
            .get("key")
            .and_then(|v| v.to_str().ok().map(String::from))
            .unwrap_or_else(|| "default".to_string());

        let cache_key = format!("{}:{}", self.config.cache_key_prefix, auth.to_str().unwrap());

        let jwks: Jwks = cache::get(&cache_key, || {
            let jwks = Jwks { key: value };
            let item = CachedItem::new(jwks, None);

            Ok(item)
        })
        .unwrap();

        Ok(Token::from_bytes(
            format!("sdk15:{}:{}", auth.to_str().unwrap(), jwks.key).into(),
        ))
    }
}
