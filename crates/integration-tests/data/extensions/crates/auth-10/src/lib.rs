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
        let auth = headers
            .get(&self.config.header_name)
            .ok_or_else(|| {
                ErrorResponse::new(StatusCode::UNAUTHORIZED)
                    .with_error(Error::new("Not passing through on my watch! SDK-10"))
            })?
            .to_str()
            .unwrap()
            .to_owned();

        let value = headers
            .get("key")
            .map(|value| value.to_str().unwrap().to_owned())
            .unwrap_or_else(|| "default".to_owned());

        let cache_key = format!("{}:{auth}", self.config.cache_key_prefix);

        let jwks: Jwks = cache::get(&cache_key, || {
            let jwks = Jwks { key: value };
            let item = CachedItem::new(jwks, None);

            Ok(item)
        })
        .unwrap();

        Ok(Token::from_bytes(format!("sdk10:{auth}:{}", jwks.key).into()))
    }
}
