use extension_catalog::ExtensionId;
use runtime::{
    auth::{AccessToken, ExtensionToken},
    error::ErrorResponse,
    extension::{AuthorizerId, ExtensionRuntime},
};
use schema::{AuthConfig, AuthProviderConfig};

use super::Runtime;

pub struct AuthExtensionService {
    authorizer_id: AuthorizerId,
    extension_id: ExtensionId,
}

impl AuthExtensionService {
    pub fn new(config: AuthConfig) -> Option<Self> {
        config.providers.iter().enumerate().find_map(|(i, provider)| {
            if let AuthProviderConfig::Extension(extension_id) = provider {
                Some(Self {
                    authorizer_id: i.into(),
                    extension_id: *extension_id,
                })
            } else {
                None
            }
        })
    }

    pub async fn authenticate<R: Runtime>(
        &self,
        runtime: &R,
        headers: http::HeaderMap,
    ) -> Result<(http::HeaderMap, AccessToken), ErrorResponse> {
        let (headers, claims) = runtime
            .extensions()
            .authenticate(self.extension_id, self.authorizer_id, headers)
            .await?;

        let token = AccessToken::Extension(ExtensionToken { claims });

        Ok((headers, token))
    }
}
