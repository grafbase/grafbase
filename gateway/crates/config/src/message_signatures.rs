use serde::de::Error;
use std::time::Duration;

#[derive(Debug, Default, serde::Deserialize, Clone, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct MessageSignaturesConfig {
    pub enabled: Option<bool>,
    pub algorithm: Option<MessageSigningAlgorithm>,
    pub key: Option<MessageSigningKey>,

    #[serde(deserialize_with = "duration_str::deserialize_option_duration")]
    pub expiry: Option<Duration>,

    pub headers: MessageSigningHeaders,
    #[serde(default = "defaults::derived_components")]
    pub derived_components: Vec<DerivedComponent>,
    #[serde(default = "defaults::signature_parameters")]
    pub signature_parameters: Vec<SignatureParameter>,
}

#[derive(Debug, serde::Deserialize, Clone, PartialEq)]
pub enum MessageSigningAlgorithm {
    HmacSha256,
    Ed25519,
    EcdsaP256,
    EcdsaP384,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageSigningKey {
    File { name: String, id: Option<String> },
    Inline { contents: String, id: Option<String> },
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
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

/// Which of the signature parameters to include in the signature
///
/// https://www.rfc-editor.org/rfc/rfc9421.html#name-signature-parameters
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignatureParameter {
    Created,
    #[serde(rename = "alg")]
    Algorithm,
    #[serde(rename = "kid")]
    KeyId,
    Nonce,
}

mod defaults {
    use super::{DerivedComponent, SignatureParameter};

    pub fn derived_components() -> Vec<DerivedComponent> {
        vec![DerivedComponent::RequestTarget]
    }

    pub fn signature_parameters() -> Vec<SignatureParameter> {
        vec![SignatureParameter::Created, SignatureParameter::Algorithm]
    }
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
                include: [],
                exclude: [],
            },
            derived_components: [
                RequestTarget,
            ],
            signature_parameters: [
                Created,
                Algorithm,
            ],
        }
        "###);
    }

    #[test]
    fn test_valid_message_signatures_config() {
        let config = indoc! {r#"
            enabled = true
            algorithm = "EcdsaP256"
            key.file = "key.file"
            key.id = "hello"
            expiry = "1s"
            headers.include = ["my-fave-header"]
            headers.exclude = ["authorization"]
            derived_components = ["request_target", "path"]
            signature_parameters = ["created"]
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
                include: [
                    "my-fave-header",
                ],
                exclude: [
                    "authorization",
                ],
            },
            derived_components: [
                RequestTarget,
                Path,
            ],
            signature_parameters: [
                Created,
            ],
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
