use std::{collections::HashSet, time::Duration};

use anyhow::Context;
use gateway_config::{
    message_signatures::{DerivedComponent, MessageSigningAlgorithm, MessageSigningKey, SignatureParameter},
    MessageSignaturesConfig,
};
use httpsig::prelude::{
    message_component::HttpMessageComponentId, HttpSignatureParams, SecretKey, SharedKey, SigningKey,
};
use httpsig_hyper::MessageSignatureReq;
use runtime::fetch::FetchError;
use tracing::Instrument;

use super::{reqwest_error_to_fetch_error, NativeFetcher};

impl NativeFetcher {
    pub async fn sign_request(
        &self,
        subgraph_name: &str,
        request: reqwest::Request,
    ) -> Result<reqwest::Request, FetchError> {
        let signature_params = self
            .subgraph_signing_parameters
            .get(subgraph_name)
            .map(Option::as_ref)
            .or(Some(self.default_signing_parameters.as_ref()))
            .flatten();

        let Some(signature_params) = signature_params else {
            return Ok(request);
        };

        let span = tracing::info_span!(target: grafbase_telemetry::span::GRAFBASE_TARGET, "http-signature");

        let mut http_request = http::Request::try_from(request).map_err(FetchError::any)?;

        let signature_parameters = signature_params
            .httpsig_params(http_request.headers())
            .map_err(FetchError::any)?;

        let sign_result = http_request
            .set_message_signature(&signature_parameters, &signature_params.key, None)
            .instrument(span)
            .await;

        if let Err(error) = sign_result {
            tracing::error!("Could not sign http request: {error}");
            return Err(FetchError::MessageSigningFailed(error.to_string()));
        }

        reqwest::Request::try_from(http_request).map_err(reqwest_error_to_fetch_error)
    }
}

#[derive(Clone)]
pub struct SigningParameters {
    key: Key,
    key_id: Option<String>,
    expiry: Option<Duration>,
    include_headers: Option<HashSet<String>>,
    exclude_headers: HashSet<String>,
    #[expect(unused)]
    derived_components: Vec<DerivedComponent>,
    #[expect(unused)]
    signature_parameters: Vec<SignatureParameter>,
}

const HOP_BY_HOP_HEADERS: &[&str] = &[
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailers",
    "transfer-encoding",
    "upgrade",
];

impl SigningParameters {
    pub fn from_config(
        config: &gateway_config::MessageSignaturesConfig,
        global: Option<&gateway_config::MessageSignaturesConfig>,
    ) -> anyhow::Result<Option<Self>> {
        let mut config = config.clone();
        if let Some(global) = global {
            config = merge_config(config, global);
        }

        let MessageSignaturesConfig {
            enabled,
            algorithm,
            key,
            expiry,
            headers,
            derived_components,
            signature_parameters,
        } = config;

        if !enabled.unwrap_or_default() {
            return Ok(None);
        }

        let Some(key) = key else { todo!("error handling") };
        let key_id = key.id().map(str::to_string);
        let key = Key::load_from(&key, algorithm)?;

        // TODO: Also add the default derived_components and/or signature_parameters

        let include_headers = headers.include.map(|headers| headers.into_iter().collect());
        let mut exclude_headers = headers
            .exclude
            .map(|headers| headers.into_iter().collect::<HashSet<_>>())
            .unwrap_or_default();

        exclude_headers.extend(HOP_BY_HOP_HEADERS.iter().map(|header| header.to_string()));

        Ok(Some(SigningParameters {
            key,
            key_id,
            expiry,
            include_headers,
            exclude_headers,
            derived_components: derived_components.unwrap_or_default(),
            signature_parameters: signature_parameters.unwrap_or_default(),
        }))
    }

    fn httpsig_params(&self, headers: &http::HeaderMap) -> anyhow::Result<HttpSignatureParams> {
        let covered_components = headers
            .iter()
            .filter(|(name, _)| self.should_include_header(name.as_str()))
            .map(|(name, _)| HttpMessageComponentId::try_from(name.as_str()))
            .collect::<Result<Vec<_>, _>>()?;

        // TODO: Handle derived_components & signature parameters

        let mut params = HttpSignatureParams::try_new(&covered_components)?;
        params.set_key_info(&self.key);
        if let Some(id) = &self.key_id {
            params.set_keyid(id);
        }

        if let Some(expiry) = self.expiry {
            params.set_expires_with_duration(Some(expiry.as_secs()));
        }

        Ok(params)
    }

    fn should_include_header(&self, name: &str) -> bool {
        if self.exclude_headers.contains(name) {
            return false;
        }

        if let Some(include_headers) = &self.include_headers {
            if include_headers.contains(name) {
                return true;
            }
        }

        true
    }
}

fn merge_config(
    mut into: gateway_config::MessageSignaturesConfig,
    from: &gateway_config::MessageSignaturesConfig,
) -> gateway_config::MessageSignaturesConfig {
    into.enabled = into.enabled.or(from.enabled);
    into.algorithm = into.algorithm.or_else(|| from.algorithm.clone());
    into.key = into.key.or_else(|| from.key.clone());
    into.expiry = into.expiry.or(from.expiry);
    into.headers.include = into.headers.include.or_else(|| from.headers.include.clone());
    into.headers.exclude = into.headers.exclude.or_else(|| from.headers.exclude.clone());
    into.derived_components = into.derived_components.or_else(|| from.derived_components.clone());
    into.signature_parameters = into.signature_parameters.or_else(|| from.signature_parameters.clone());

    into
}

#[derive(Clone)]
pub enum Key {
    Secret(httpsig::prelude::SecretKey),
    Shared(httpsig::prelude::SharedKey),
}

impl Key {
    pub fn load_from(key: &MessageSigningKey, algorithm: Option<MessageSigningAlgorithm>) -> anyhow::Result<Self> {
        match key {
            MessageSigningKey::File { name, .. } => Self::from_file(name, algorithm),
            MessageSigningKey::Inline { contents, .. } => Self::from_string(contents, algorithm),
        }
    }

    pub fn from_string(contents: &str, algorithm: Option<MessageSigningAlgorithm>) -> anyhow::Result<Self> {
        let algorithm = algorithm.unwrap_or(MessageSigningAlgorithm::HmacSha256);
        match algorithm {
            MessageSigningAlgorithm::HmacSha256 => Ok(Key::Shared(
                SharedKey::from_base64(contents).context("when parsing hmac-sha256 key from inline base64 string")?,
            )),
            _ => Err(anyhow::anyhow!(
                "only hmac-sha256 message signatures are supported when providing an inline key"
            )),
        }
    }

    pub fn from_file(name: &str, algorithm: Option<MessageSigningAlgorithm>) -> anyhow::Result<Self> {
        let data = std::fs::read_to_string(name)?;

        if let Some(MessageSigningAlgorithm::HmacSha256) = algorithm {
            return Ok(Self::Shared(SharedKey::from_base64(&data).with_context(|| {
                format!("when parsing base64 encoded hmac-sha256 key from {name}")
            })?));
        }

        let key = SecretKey::from_pem(&data).with_context(|| format!("error when parsing secret key from {name}"))?;

        match (&key, algorithm) {
            (_, None)
            | (SecretKey::Ed25519(_), Some(MessageSigningAlgorithm::Ed25519))
            | (SecretKey::EcdsaP256Sha256(_), Some(MessageSigningAlgorithm::EcdsaP256))
            | (SecretKey::EcdsaP384Sha384(_), Some(MessageSigningAlgorithm::EcdsaP384)) => {}
            (_, Some(algorithm)) => {
                return Err(anyhow::anyhow!(
                    "you requested {algorithm} message signing but {name} contains a {} key",
                    key.alg().as_str()
                ))
            }
        }

        Ok(Self::Secret(key))
    }
}

impl SigningKey for Key {
    fn sign(&self, data: &[u8]) -> httpsig_hyper::prelude::HttpSigResult<Vec<u8>> {
        match self {
            Key::Secret(secret_key) => secret_key.sign(data),
            Key::Shared(shared_key) => shared_key.sign(data),
        }
    }

    fn key_id(&self) -> String {
        match self {
            Key::Secret(secret_key) => secret_key.key_id(),
            Key::Shared(shared_key) => shared_key.key_id(),
        }
    }

    fn alg(&self) -> httpsig_hyper::prelude::AlgorithmName {
        match self {
            Key::Secret(secret_key) => secret_key.alg(),
            Key::Shared(shared_key) => shared_key.alg(),
        }
    }
}
