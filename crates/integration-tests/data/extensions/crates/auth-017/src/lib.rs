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
                .with_error(Error::new("Not passing through on my watch! SDK-017"))
                .with_header("Www-Authenticate", b"Bearer test_author=grafbase")
                .unwrap()
        })?;

        let auth = auth.to_str().unwrap();

        let value = headers
            .get("value")
            .map(|value| value.to_str().unwrap().to_owned())
            .unwrap_or_else(|| String::from("default"));

        let cache_key = format!("{}:{auth}", self.config.cache_key_prefix);

        let jwks: Jwks = cache::get(&cache_key, || {
            let jwks = Jwks { key: value };
            let item = CachedItem::new(jwks, None);

            Ok(item)
        })
        .unwrap();

        Ok(Token::from_bytes(format!("sdk017:{auth}:{}", jwks.key).into()))
    }
}
