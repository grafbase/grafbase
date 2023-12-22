use openidconnect::{
    core::{CoreClient, CoreProviderMetadata},
    reqwest::async_http_client,
    ClientId, ClientSecret, IssuerUrl,
};
use ory_client::apis::configuration::Configuration;

// Defined in docker-compose.yml
pub const ISSUER: &str = "http://127.0.0.1:4444";
pub const JWKS_URI: &str = "http://127.0.0.1:4444/.well-known/jwks.json";
pub const JWKS_URI_2: &str = "http://127.0.0.1:4454/.well-known/jwks.json";
pub const AUDIENCE: &str = "integration-tests";
pub const OTHER_AUDIENCE: &str = "other-audience";
const HYDRA_ADMIN_URL: &str = "http://127.0.0.1:4445";
// Second provider
pub const ISSUER_2: &str = "http://127.0.0.1:4454";
const HYDRA_2_ADMIN_URL: &str = "http://127.0.0.1:4455";

pub struct OryHydraOpenIDProvider {
    issuer: IssuerUrl,
    ory_config: Configuration,
}

impl Default for OryHydraOpenIDProvider {
    fn default() -> Self {
        Self {
            issuer: IssuerUrl::new(ISSUER.to_string()).unwrap(),
            ory_config: Configuration {
                base_path: HYDRA_ADMIN_URL.to_string(),
                ..Default::default()
            },
        }
    }
}

impl OryHydraOpenIDProvider {
    pub fn second_provider() -> OryHydraOpenIDProvider {
        Self {
            issuer: IssuerUrl::new(ISSUER_2.to_string()).unwrap(),
            ory_config: Configuration {
                base_path: HYDRA_2_ADMIN_URL.to_string(),
                ..Default::default()
            },
        }
    }

    pub async fn create_client(&self) -> CoreClient {
        let resp = ory_client::apis::o_auth2_api::create_o_auth2_client(
            &self.ory_config,
            &ory_client::models::OAuth2Client {
                access_token_strategy: Some("jwt".into()),
                grant_types: Some(vec!["client_credentials".into()]),
                // whitelisted audiences
                audience: Some(vec![AUDIENCE.into(), OTHER_AUDIENCE.into()]),
                ..ory_client::models::OAuth2Client::new()
            },
        )
        .await
        .unwrap();

        let provider_metadata = CoreProviderMetadata::discover_async(self.issuer.clone(), async_http_client)
            .await
            .unwrap();

        CoreClient::from_provider_metadata(
            provider_metadata,
            ClientId::new(resp.client_id.unwrap()),
            Some(ClientSecret::new(resp.client_secret.unwrap())),
        )
    }
}

#[async_trait::async_trait]
pub trait CoreClientExt {
    fn client(&self) -> &CoreClient;
    async fn get_access_token_with_client_credentials(&self, extra_params: &[(&str, &str)]) -> String {
        use openidconnect::OAuth2TokenResponse;

        let mut request = self.client().exchange_client_credentials();
        for (key, value) in extra_params {
            request = request.add_extra_param(*key, *value);
        }
        request
            .request_async(openidconnect::reqwest::async_http_client)
            .await
            .unwrap()
            .access_token()
            .secret()
            .clone()
    }
}

impl CoreClientExt for CoreClient {
    fn client(&self) -> &CoreClient {
        self
    }
}
