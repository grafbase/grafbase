use std::sync::Arc;

use extension_catalog::ExtensionId;
use runtime::{
    auth::{AccessToken, ExtensionToken},
    error::ErrorResponse,
    extension::{AuthorizerId, ExtensionRuntime},
};
use schema::{AuthConfig, AuthProviderConfig};

use super::Runtime;

pub struct AuthExtensionService {
    authorizers: Vec<(AuthorizerId, ExtensionId)>,
}

impl AuthExtensionService {
    pub fn new(config: AuthConfig) -> Option<Self> {
        let mut authorizers = Vec::new();

        for (i, provider) in config.providers.iter().enumerate() {
            if let AuthProviderConfig::Extension(extension_id) = provider {
                authorizers.push((i.into(), *extension_id));
            }
        }

        if authorizers.is_empty() {
            None
        } else {
            Some(Self { authorizers })
        }
    }

    pub async fn authenticate<R: Runtime>(
        &self,
        runtime: &R,
        headers: http::HeaderMap,
    ) -> Result<(http::HeaderMap, AccessToken), ErrorResponse> {
        let headers = Arc::new(headers);
        let mut last_result = None;

        for (authorizer_id, extension_id) in &self.authorizers {
            match authenticate(runtime, *extension_id, *authorizer_id, headers.clone()).await {
                Ok(result) => {
                    let headers = Arc::into_inner(headers).expect("we had more than one reference to headers");

                    return Ok((headers, result));
                }
                Err(err) => {
                    last_result = Some(Err(err));
                }
            }
        }

        match last_result {
            Some(result) => result,
            None => {
                let headers = Arc::into_inner(headers).expect("we had more than one reference to headers");

                Ok((headers, AccessToken::Anonymous))
            }
        }
    }
}

async fn authenticate<R: Runtime>(
    runtime: &R,
    extension_id: ExtensionId,
    authorizer_id: AuthorizerId,
    headers: Arc<http::HeaderMap>,
) -> Result<AccessToken, ErrorResponse> {
    let claims = runtime
        .extensions()
        .authenticate(extension_id, authorizer_id, headers)
        .await?;

    let token = AccessToken::Extension(ExtensionToken { claims });

    Ok(token)
}
