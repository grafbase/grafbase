use serde::de::Error;
use std::{fmt, time::Duration};

#[derive(Debug, Default, serde::Deserialize, Clone, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct MessageSignaturesConfig {
    pub enabled: Option<bool>,
    pub algorithm: Option<MessageSigningAlgorithm>,
    pub key: Option<MessageSigningKey>,

    #[serde(deserialize_with = "duration_str::deserialize_option_duration")]
    pub expiry: Option<Duration>,

    pub headers: MessageSigningHeaders,
    pub derived_components: Option<Vec<DerivedComponent>>,
    pub signature_parameters: Option<Vec<SignatureParameter>>,
}

/// Name conventions follow [Section 6.2.2, RFC9421](https://datatracker.ietf.org/doc/html/rfc9421#section-6.2.2)
#[derive(Debug, serde::Deserialize, Clone, PartialEq)]
pub enum MessageSigningAlgorithm {
    #[serde(rename = "hmac-sha256")]
    HmacSha256,
    #[serde(rename = "ed25519")]
    Ed25519,
    #[serde(rename = "ecdsa-p256-sha256")]
    EcdsaP256,
    #[serde(rename = "ecdsa-p384-sha384")]
    EcdsaP384,
}

impl fmt::Display for MessageSigningAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            MessageSigningAlgorithm::HmacSha256 => "hmac-sha256",
            MessageSigningAlgorithm::Ed25519 => "ed25516",
            MessageSigningAlgorithm::EcdsaP256 => "ecdsa-p256-sha256",
            MessageSigningAlgorithm::EcdsaP384 => "ecdsa-p384-sha384",
        };

        write!(f, "{str}")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageSigningKey {
    File { name: String, id: Option<String> },
    Inline { contents: String, id: Option<String> },
}

impl MessageSigningKey {
    pub fn id(&self) -> Option<&str> {
        match self {
            MessageSigningKey::File { id, .. } => id.as_deref(),
            MessageSigningKey::Inline { id, .. } => id.as_deref(),
        }
    }
}

impl<'de> serde::Deserialize<'de> for MessageSigningKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct RawStruct {
            file: Option<String>,
            inline: Option<String>,
            id: Option<String>,
        }
        let RawStruct { file, inline, id } = RawStruct::deserialize(deserializer)?;

        match (file, inline) {
            (None, None) => Err(D::Error::custom(
                "one of raw or file must be provided in the message signing key",
            )),
            (Some(_), Some(_)) => Err(D::Error::custom(
                "raw and file may not both be provided in a message signing key",
            )),
            (Some(name), None) => Ok(MessageSigningKey::File { name, id }),
            (None, Some(contents)) => Ok(MessageSigningKey::Inline { contents, id }),
        }
    }
}

/// Which of the derived components to include in the signature:
///
/// https://www.rfc-editor.org/rfc/rfc9421.html#name-derived-components
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DerivedComponent {
    Method,
    TargetUri,
    Authority,
    Scheme,
    RequestTarget,
    Path,
}

/// Which headers to include/exclude in the signature
#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MessageSigningHeaders {
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
}

/// Which of the signature parameters to include in the signature
///
/// https://www.rfc-editor.org/rfc/rfc9421.html#name-signature-parameters
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignatureParameter {
    Nonce,
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_default_message_signatures_config() {
        let config = toml::from_str::<MessageSignaturesConfig>("").unwrap();

        insta::assert_debug_snapshot!(&config, @r###"
        MessageSignaturesConfig {
            enabled: None,
            algorithm: None,
            key: None,
            expiry: None,
            headers: MessageSigningHeaders {
                include: None,
                exclude: None,
            },
            derived_components: None,
            signature_parameters: None,
        }
        "###);
    }

    #[test]
    fn test_valid_message_signatures_config() {
        let config = indoc! {r#"
            enabled = true
            algorithm = "ecdsa-p256-sha256"
            key.file = "key.file"
            key.id = "hello"
            expiry = "1s"
            headers.include = ["my-fave-header"]
            headers.exclude = ["authorization"]
            derived_components = ["request_target", "path"]
            signature_parameters = ["nonce"]
        "#};

        let config = toml::from_str::<MessageSignaturesConfig>(config).unwrap();

        insta::assert_debug_snapshot!(&config, @r###"
        MessageSignaturesConfig {
            enabled: Some(
                true,
            ),
            algorithm: Some(
                EcdsaP256,
            ),
            key: Some(
                File {
                    name: "key.file",
                    id: Some(
                        "hello",
                    ),
                },
            ),
            expiry: Some(
                1s,
            ),
            headers: MessageSigningHeaders {
                include: Some(
                    [
                        "my-fave-header",
                    ],
                ),
                exclude: Some(
                    [
                        "authorization",
                    ],
                ),
            },
            derived_components: Some(
                [
                    RequestTarget,
                    Path,
                ],
            ),
            signature_parameters: Some(
                [
                    Nonce,
                ],
            ),
        }
        "###);
    }

    #[test]
    fn test_inline_key() {
        let config = indoc! {r#"
            key.file = "super-secret-key"
            key.id = "hello"
        "#};

        let config = toml::from_str::<MessageSignaturesConfig>(config).unwrap();

        insta::assert_debug_snapshot!(config.key, @r###"
        Some(
            File {
                name: "super-secret-key",
                id: Some(
                    "hello",
                ),
            },
        )
        "###);
    }

    #[test]
    fn test_inline_and_file_key() {
        let config = indoc! {r#"
            key.file = "super-secret-key"
            key.inline = "hello"
        "#};

        let result = toml::from_str::<MessageSignaturesConfig>(config);

        insta::assert_debug_snapshot!(result, @r###"
        Err(
            Error {
                inner: Error {
                    inner: TomlError {
                        message: "raw and file may not both be provided in a message signing key",
                        raw: Some(
                            "key.file = \"super-secret-key\"\nkey.inline = \"hello\"\n",
                        ),
                        keys: [
                            "key",
                        ],
                        span: Some(
                            0..3,
                        ),
                    },
                },
            },
        )
        "###);
    }

    #[test]
    fn test_empty_key() {
        let config = indoc! {r#"
            key = {}
        "#};

        let result = toml::from_str::<MessageSignaturesConfig>(config);

        insta::assert_debug_snapshot!(result, @r###"
        Err(
            Error {
                inner: Error {
                    inner: TomlError {
                        message: "one of raw or file must be provided in the message signing key",
                        raw: Some(
                            "key = {}\n",
                        ),
                        keys: [
                            "key",
                        ],
                        span: Some(
                            6..8,
                        ),
                    },
                },
            },
        )
        "###);
    }
}
