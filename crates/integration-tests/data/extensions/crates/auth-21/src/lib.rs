use grafbase_sdk::{
    AuthenticationExtension,
    host_io::{cache::Cache, http::StatusCode},
    types::{Configuration, Error, ErrorResponse, Headers, PublicMetadataEndpoint, RequestContext, Token},
};

#[derive(AuthenticationExtension)]
struct CachingProvider {
    config: Config,
    cache: Cache,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
enum Config {
    Provider(ProviderConfig),
    ErrorWithContext,
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
        let config: Config = config.deserialize().unwrap_or(Config::Provider(Default::default()));

        Ok(Self {
            config,
            cache: Cache::builder("jwks", 128).build(),
        })
    }

    fn authenticate(&mut self, ctx: &RequestContext, headers: &Headers) -> Result<Token, ErrorResponse> {
        match &self.config {
            Config::Provider(config) => {
                let auth = headers.get(&config.header_name).ok_or_else(|| {
                    ErrorResponse::new(StatusCode::UNAUTHORIZED)
                        .with_error(Error::new("Not passing through on my watch! SDK-21"))
                        .with_header("Www-Authenticate", b"Bearer test_author=grafbase")
                        .unwrap()
                })?;

                let auth = auth.to_str().unwrap();

                let value = headers
                    .get("key")
                    .map(|value| value.to_str().unwrap().to_owned())
                    .unwrap_or_else(|| String::from("default"));

                let cache_key = format!("{}:{auth}", config.cache_key_prefix);

                let jwks: Jwks = serde_json::from_slice(
                    &self
                        .cache
                        .get_or_insert(&cache_key, || {
                            serde_json::to_vec(&Jwks { key: value }).map(|bytes| ((), bytes))
                        })
                        .unwrap()
                        .1,
                )
                .unwrap();

                Ok(Token::from_bytes(format!("sdk21:{auth}:{}", jwks.key).into()))
            }
            Config::ErrorWithContext => Err(ErrorResponse::new(StatusCode::UNAUTHORIZED)
                .with_error(Error::new(String::from_utf8_lossy(&ctx.hooks_context()).into_owned()))),
        }
    }

    fn public_metadata(&mut self) -> Result<Vec<PublicMetadataEndpoint>, Error> {
        match &self.config {
            Config::Provider(config) => {
                let Some(oauth) = &config.oauth else {
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
            _ => Ok(Vec::new()),
        }
    }
}
