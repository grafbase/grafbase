use openidconnect::{
    core::{CoreClient, CoreProviderMetadata},
    ClientId, ClientSecret, EndpointMaybeSet, EndpointNotSet, EndpointSet, IssuerUrl,
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
pub const READ_SCOPE: &str = "read";
pub const WRITE_SCOPE: &str = "write";

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

    pub async fn create_client(
        &self,
    ) -> CoreClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet, EndpointMaybeSet> {
        let resp = ory_client::apis::o_auth2_api::create_o_auth2_client(
            &self.ory_config,
            &ory_client::models::OAuth2Client {
                access_token_strategy: Some("jwt".into()),
                grant_types: Some(vec!["client_credentials".into()]),
                // Allowed audiences
                audience: Some(vec![AUDIENCE.into(), OTHER_AUDIENCE.into()]),
                // Allowed scopes
                scope: Some(format!("{READ_SCOPE} {WRITE_SCOPE}")),
                ..ory_client::models::OAuth2Client::new()
            },
        )
        .await
        .unwrap();

        let reqwest_client = reqwest::Client::new();
        let provider_metadata = CoreProviderMetadata::discover_async(self.issuer.clone(), &reqwest_client)
            .await
            .unwrap();
        let token_uri = provider_metadata.token_endpoint().expect("must be defined").clone();

        CoreClient::from_provider_metadata(
            provider_metadata,
            ClientId::new(resp.client_id.unwrap()),
            Some(ClientSecret::new(resp.client_secret.unwrap())),
        )
        .set_token_uri(token_uri)
        // It is silly that we need to explicitly pass in the token URL, but that's the only way to ensure the returned
        // client type's has `HasTokenUrl` set to `EndpointSet`, as `from_provider_metadata()` assumes the metadata passed in
        // may be missing it.
    }
}

#[allow(async_fn_in_trait)]
pub trait CoreClientExt {
    async fn get_access_token_with_client_credentials(&self, extra_params: &[(&str, &str)]) -> String;
}

/// Methods requiring a token endpoint.
impl<
        AC,
        AD,
        GC,
        JE,
        JS,
        JT,
        K,
        P,
        TE,
        TR,
        TIR,
        RT,
        TRE,
        HasAuthUrl,
        HasDeviceAuthUrl,
        HasIntrospectionUrl,
        HasRevocationUrl,
        HasUserInfoUrl,
    > CoreClientExt
    for openidconnect::Client<
        AC,
        AD,
        GC,
        JE,
        K,
        P,
        TE,
        TR,
        TIR,
        RT,
        TRE,
        HasAuthUrl,
        HasDeviceAuthUrl,
        HasIntrospectionUrl,
        HasRevocationUrl,
        EndpointSet,
        HasUserInfoUrl,
    >
where
    AC: openidconnect::AdditionalClaims,
    AD: openidconnect::AuthDisplay,
    GC: openidconnect::GenderClaim,
    JT: openidconnect::JsonWebKeyType,
    JE: openidconnect::JweContentEncryptionAlgorithm<KeyType = JT>,
    JS: openidconnect::JwsSigningAlgorithm<KeyType = JT>,
    K: openidconnect::JsonWebKey<SigningAlgorithm = JS>,
    P: openidconnect::AuthPrompt,
    TE: openidconnect::ErrorResponse + 'static,
    TR: openidconnect::TokenResponse<AC, GC, JE, JS>,
    TIR: openidconnect::TokenIntrospectionResponse,
    RT: openidconnect::RevocableToken,
    TRE: openidconnect::ErrorResponse + 'static,
    HasAuthUrl: openidconnect::EndpointState,
    HasDeviceAuthUrl: openidconnect::EndpointState,
    HasIntrospectionUrl: openidconnect::EndpointState,
    HasRevocationUrl: openidconnect::EndpointState,
    HasUserInfoUrl: openidconnect::EndpointState,
{
    async fn get_access_token_with_client_credentials(&self, extra_params: &[(&str, &str)]) -> String {
        let reqwest_client = reqwest::Client::new();
        let mut request = self.exchange_client_credentials();
        for (key, value) in extra_params {
            request = request.add_extra_param(*key, *value);
        }
        request
            .request_async(&reqwest_client)
            .await
            .unwrap()
            .access_token()
            .secret()
            .clone()
    }
}
