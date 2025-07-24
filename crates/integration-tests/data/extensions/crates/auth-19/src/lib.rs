use grafbase_sdk::{
    AuthenticationExtension,
    host_io::{cache::Cache, http::StatusCode},
    types::{Configuration, Error, ErrorResponse, Headers, PublicMetadataEndpoint, Token},
};

#[derive(AuthenticationExtension)]
struct CachingProvider {
    config: ProviderConfig,
    cache: Cache,
}

#[derive(Debug, serde::Deserialize)]
#[serde(default)]
struct ProviderConfig {
    header_name: String,
    cache_key_prefix: String,
    #[serde(default)]
    oauth: Option<OAuthConfig>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            header_name: "Authorization".to_owned(),
            cache_key_prefix: "test".to_owned(),
            oauth: Default::default(),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
struct OAuthConfig {
    resource: String,
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
            cache: Cache::builder("jwks", 128).build(),
        })
    }

    fn authenticate(&mut self, headers: &Headers) -> Result<Token, ErrorResponse> {
        let auth = headers.get(&self.config.header_name).ok_or_else(|| {
            ErrorResponse::new(StatusCode::UNAUTHORIZED)
                .with_error(Error::new("Not passing through on my watch! SDK-19"))
                .with_header("Www-Authenticate", b"Bearer test_author=grafbase")
                .unwrap()
        })?;

        let auth = auth.to_str().unwrap();

        let value = headers
            .get("key")
            .map(|value| value.to_str().unwrap().to_owned())
            .unwrap_or_else(|| String::from("default"));

        let cache_key = format!("{}:{auth}", self.config.cache_key_prefix);

        let jwks: Jwks = serde_json::from_slice(
            &self
                .cache
                .get_or_insert(&cache_key, || serde_json::to_vec(&Jwks { key: value }))
                .unwrap(),
        )
        .unwrap();

        Ok(Token::from_bytes(format!("sdk19:{auth}:{}", jwks.key).into()))
    }

    fn public_metadata(&mut self) -> Result<Vec<PublicMetadataEndpoint>, Error> {
        let Some(oauth) = &self.config.oauth else {
            return Ok(vec![]);
        };

        let mut response_headers = Headers::new();
        response_headers.append("x-test", "works");

        Ok(vec![
            PublicMetadataEndpoint::new(
                "/.well-known/protected-resource".to_owned(),
                serde_json::to_vec(&serde_json::json!({
                    "resource": oauth.resource,
                }))
                .unwrap(),
            )
            .with_headers(response_headers),
        ])
    }
}
