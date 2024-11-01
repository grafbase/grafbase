use p256 as _; // Don't use this directly but we need to enable its JWK feature
use p384 as _;
use serde::Deserialize;
use serde_json::json; // Don't use this directly but we need to enable its JWK feature

use std::{collections::HashSet, time::Duration};

use anyhow::{anyhow, Context};
use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
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
    derived_components: Vec<DerivedComponent>,
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
            derived_components: derived_components.unwrap_or_else(|| vec![DerivedComponent::RequestTarget]),
            signature_parameters: signature_parameters.unwrap_or_default(),
        }))
    }

    fn httpsig_params(&self, headers: &http::HeaderMap) -> anyhow::Result<HttpSignatureParams> {
        let mut covered_components = headers
            .iter()
            .filter(|(name, _)| self.should_include_header(name.as_str()))
            .map(|(name, _)| HttpMessageComponentId::try_from(name.as_str()))
            .collect::<Result<Vec<_>, _>>()?;

        covered_components.extend(
            self.derived_components
                .iter()
                .map(derived_component_to_message_component),
        );

        let mut params = HttpSignatureParams::try_new(&covered_components)?;
        params.set_key_info(&self.key);
        if matches!(self.key, Key::Shared(_)) {
            // I am _pretty sure_ set_key_info just makes up a nonsense id for shared keys
            // so lets clear it out.  Can remove this if I turn out to be wrong.
            params.keyid = None;
        }
        if let Some(id) = &self.key_id {
            params.set_keyid(id);
        }

        for param in &self.signature_parameters {
            match param {
                SignatureParameter::Nonce => {
                    params.set_random_nonce();
                }
            }
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
                SharedKey::from_base64(contents)
                    .context("could not parse hmac-sha256 key from inline base64 string")?,
            )),
            _ => Err(anyhow::anyhow!(
                "only hmac-sha256 message signatures are supported when providing an inline key"
            )),
        }
    }

    pub fn from_file(name: &str, algorithm: Option<MessageSigningAlgorithm>) -> anyhow::Result<Self> {
        let data = std::fs::read_to_string(name)?;

        let key = if data.trim_start().starts_with('{') {
            // If the file is very clearly a JSON object we'll treat it like a JWK
            parse_jwk(&data, name)?
        } else if algorithm == Some(MessageSigningAlgorithm::HmacSha256) {
            Key::Shared(
                SharedKey::from_base64(&data)
                    .with_context(|| format!("could not parse base64 encoded hmac-sha256 key from {name}"))?,
            )
        } else {
            Key::Secret(SecretKey::from_pem(&data).with_context(|| {
                format!("could not parse a key from {name} (expected a PEM file containing a PKCS#8 format key)")
            })?)
        };

        match (&key, algorithm) {
            (_, None)
            | (Key::Shared(_), Some(MessageSigningAlgorithm::HmacSha256))
            | (Key::Secret(SecretKey::Ed25519(_)), Some(MessageSigningAlgorithm::Ed25519))
            | (Key::Secret(SecretKey::EcdsaP256Sha256(_)), Some(MessageSigningAlgorithm::EcdsaP256))
            | (Key::Secret(SecretKey::EcdsaP384Sha384(_)), Some(MessageSigningAlgorithm::EcdsaP384)) => {}
            (_, Some(algorithm)) => {
                return Err(anyhow::anyhow!(
                    "you requested {algorithm} message signing but {name} contains a {} key",
                    key.alg().as_str()
                ))
            }
        }

        Ok(key)
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

#[derive(serde::Deserialize)]
pub struct Jwk {
    #[serde(rename = "kty")]
    key_type: String,
    #[serde(rename = "crv")]
    curve: Option<String>,
    #[serde(rename = "d")]
    private_key: Option<String>,
    #[serde(rename = "alg")]
    algorithm: Option<String>,
    #[serde(rename = "k")]
    shared_key: Option<String>,

    x: Option<String>,
    y: Option<String>,
}

// TODO: return the kid from this as well...
fn parse_jwk(data: &str, filename: &str) -> anyhow::Result<Key> {
    let jwk = serde_json::from_str::<Jwk>(data)
        .with_context(|| format!("{filename} looked like a JWK but we could not parse it as one"))?;

    match (jwk.key_type.as_str(), jwk.curve.as_deref()) {
        // Values for the ecdsa curves as defined here https://datatracker.ietf.org/doc/html/rfc7518#section-6.2
        ("EC", Some("P-256")) => {
            validate_private_key(&jwk, filename)?;
            Ok(Key::Secret(SecretKey::EcdsaP256Sha256(
                ec_key_from_jwk(jwk)
                    .with_context(|| format!("could not load P-256 private key from JWK {filename}"))?,
            )))
        }
        ("EC", Some("P-384")) => {
            validate_private_key(&jwk, filename)?;
            Ok(Key::Secret(SecretKey::EcdsaP384Sha384(
                ec_key_from_jwk(jwk)
                    .with_context(|| format!("could not load P-384 private key from JWK {filename}"))?,
            )))
        }

        // Ed25519 isn't in the original JWK spec, but found the values
        // here: https://datatracker.ietf.org/doc/html/rfc8037#section-2
        ("OKP", Some("Ed25519")) => parse_ed25519(&jwk, filename),

        ("oct", _) if jwk.algorithm.as_deref() == Some("HS256") => {
            let Some(key_b64) = jwk.shared_key.as_deref() else {
                return Err(anyhow::anyhow!("The k field is missing from the JWK in {filename}"));
            };

            let key = BASE64_URL_SAFE_NO_PAD
                .decode(key_b64)
                .with_context(|| format!("could not base64 decode the key in {filename}"))?;

            Ok(Key::Shared(httpsig::prelude::SharedKey::HmacSha256(key)))
        }
        ("oct", _) => Err(anyhow::anyhow!(
            "The {:?} algorithm found in {filename} is not supported",
            jwk.algorithm
        )),
        ("EC", Some(crv)) | ("OKP", Some(crv)) => Err(anyhow::anyhow!("the {crv} JWK curve is not supported")),

        (kty, _) => Err(anyhow::anyhow!("{kty} JWKs are not supported")),
    }
}

fn parse_ed25519(jwk: &Jwk, filename: &str) -> Result<Key, anyhow::Error> {
    let private_b64 = validate_private_key(jwk, filename)?;
    let Some(public_b64) = &jwk.x else {
        return Err(anyhow::anyhow!("The Ed25519 JWK in {filename} is missing a public key"));
    };
    let Some((private, public)) = BASE64_URL_SAFE_NO_PAD
        .decode(private_b64)
        .ok()
        .zip(BASE64_URL_SAFE_NO_PAD.decode(public_b64).ok())
    else {
        return Err(anyhow::anyhow!("Couldnt base64 decode the keys in {filename}"));
    };

    let mut key = [0; ed25519_compact::SecretKey::BYTES];
    if private.len() + public.len() != key.len() {
        return Err(anyhow::anyhow!(
            "The public & private keys in {filename} dont have the expected length"
        ));
    }
    key[..ed25519_compact::PublicKey::BYTES].copy_from_slice(&private);
    key[ed25519_compact::PublicKey::BYTES..].copy_from_slice(&public);

    Ok(Key::Secret(SecretKey::Ed25519(ed25519_compact::SecretKey::new(key))))
}

fn validate_private_key<'a>(jwk: &'a Jwk, filename: &str) -> anyhow::Result<&'a str> {
    jwk.private_key
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("The JWK in {filename} does not contain a private key"))
}

fn ec_key_from_jwk<C>(jwk: Jwk) -> anyhow::Result<elliptic_curve::SecretKey<C>>
where
    C: elliptic_curve::Curve + elliptic_curve::JwkParameters + elliptic_curve::sec1::ValidatePublicKey,
    elliptic_curve::FieldBytesSize<C>: elliptic_curve::sec1::ModulusSize,
{
    let ec_jwk = elliptic_curve::JwkEcKey::try_from(jwk)?;
    Ok(elliptic_curve::SecretKey::from_jwk(&ec_jwk)?)
}

fn derived_component_to_message_component(component: &DerivedComponent) -> HttpMessageComponentId {
    HttpMessageComponentId::try_from(match component {
        DerivedComponent::Method => "@method",
        DerivedComponent::TargetUri => "@target-uri",
        DerivedComponent::Authority => "@authority",
        DerivedComponent::Scheme => "@scheme",
        DerivedComponent::RequestTarget => "@request-target",
        DerivedComponent::Path => "@path",
    })
    .expect("these components are hard coded so shouldnt fail to convert")
}

// JwkEcKey has a Deserialize impl, but it fails if there's any object keys that it
// doesn't expect in there (and it doesn't expect a whole load of fairly standard keys)
// So rather than relying on that Deserialize impl lets just convert it ourselves
//
// We should be able to get rid of this once https://github.com/RustCrypto/traits/pull/1547
// is merged, but who knows when that'll be.
impl TryFrom<Jwk> for elliptic_curve::JwkEcKey {
    type Error = anyhow::Error;

    fn try_from(value: Jwk) -> Result<Self, Self::Error> {
        let Jwk {
            key_type,
            curve,
            private_key,
            x,
            y,
            ..
        } = value;

        let curve = curve.ok_or_else(|| anyhow!("JWK was missing the crv field"))?;
        let private_key = private_key.ok_or_else(|| anyhow!("JWK was missing the d field"))?;
        let x = x.ok_or_else(|| anyhow!("JWK was missing the x field"))?;
        let y = y.ok_or_else(|| anyhow!("JWK was missing the y field"))?;

        // To make matters worse, none of the fields on JwkEcKey are public.
        // So I guess we'll have to build a fake JSON object and then use
        // the Deserialize impl. This API is a huge pain in the ass.
        Ok(elliptic_curve::JwkEcKey::deserialize(json!({
            "kty": key_type,
            "crv": curve,
            "x": x,
            "y": y,
            "d": private_key
        }))?)
    }
}
