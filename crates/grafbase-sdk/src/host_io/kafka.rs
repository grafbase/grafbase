//! # Kafka Producer Module
//!
//!
//! This module provides a high-level Rust API for creating and configuring Kafka producers.
//! It wraps the lower-level WIT (WebAssembly Interface Types) bindings to offer a more
//! ergonomic interface for publishing messages to Kafka topics.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! # use grafbase_sdk::{SdkError, host_io::kafka::{self, KafkaProducerConfig}};
//!
//! # fn main() -> Result<(), SdkError> {
//! // Create a basic producer configuration
//! let config = KafkaProducerConfig::default();
//!
//! // Connect to Kafka
//! let producer = kafka::producer(
//!     "my-producer",
//!     ["localhost:9092"],
//!     "my-topic",
//!     config
//! )?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Authentication
//!
//! The module supports multiple authentication methods:
//!
//! - **SASL/PLAIN**: Username and password authentication
//! - **SASL/SCRAM**: SHA256 and SHA512 SCRAM mechanisms
//! - **mTLS**: Mutual TLS with client certificates
//!
//! ## TLS Configuration
//!
//! TLS connections can be configured to use either the system's certificate authority
//! store or a custom CA certificate file.

mod consumer;
mod producer;

pub use consumer::{KafkaConsumer, KafkaConsumerConfig, KafkaMessage};
pub use producer::{KafkaBatchConfig, KafkaProducer, KafkaProducerCompression, KafkaProducerConfig};

use crate::{SdkError, wit};
use std::path::{Path, PathBuf};

/// Connects to Kafka servers and creates a new Kafka producer.
pub fn producer(
    name: &str,
    servers: impl IntoIterator<Item = impl ToString>,
    topic: &str,
    config: KafkaProducerConfig,
) -> Result<KafkaProducer, SdkError> {
    let servers: Vec<_> = servers.into_iter().map(|s| s.to_string()).collect();
    let config = config.into();
    let producer = wit::KafkaProducer::connect(name, &servers, topic, &config)?;

    Ok(KafkaProducer { inner: producer })
}

/// Connects to Kafka servers and creates a new Kafka consumer.
pub fn consumer(
    servers: impl IntoIterator<Item = impl ToString>,
    topic: &str,
    config: KafkaConsumerConfig,
) -> Result<KafkaConsumer, SdkError> {
    let servers: Vec<_> = servers.into_iter().map(|s| s.to_string()).collect();
    let config = config.into();
    let consumer = wit::KafkaConsumer::connect(&servers, topic, &config)?;

    Ok(KafkaConsumer { inner: consumer })
}

/// TLS configuration options for Kafka connections.
pub enum KafkaTlsConfig {
    /// Use the system's default certificate authority store
    SystemCa,
    /// Use a custom certificate authority from the specified file path
    CustomCa(PathBuf),
}

impl KafkaTlsConfig {
    /// Creates a TLS configuration that uses the system's default certificate authority store.
    pub fn system_ca() -> Self {
        Self::SystemCa
    }

    /// Creates a TLS configuration that uses a custom certificate authority from a file.
    ///
    /// # Arguments
    ///
    /// * `ca_cert_path` - Path to the custom certificate authority file
    pub fn custom_ca(ca_cert_path: impl AsRef<Path>) -> Self {
        Self::CustomCa(ca_cert_path.as_ref().to_path_buf())
    }
}

impl From<KafkaTlsConfig> for wit::KafkaTlsConfig {
    fn from(value: KafkaTlsConfig) -> Self {
        match value {
            KafkaTlsConfig::SystemCa => wit::KafkaTlsConfig::SystemCa,
            KafkaTlsConfig::CustomCa(path) => wit::KafkaTlsConfig::CustomCa(path.to_string_lossy().to_string()),
        }
    }
}

enum KafkaAuthenticationInner {
    SaslPlain(wit::KafkaSaslPlainAuth),
    SaslScram(wit::KafkaSaslScramAuth),
    Mtls(wit::KafkaMtlsAuth),
}

/// Authentication configuration for Kafka connections.
pub struct KafkaAuthentication {
    inner: KafkaAuthenticationInner,
}

impl KafkaAuthentication {
    /// Creates a SASL/PLAIN authentication configuration.
    ///
    /// # Arguments
    ///
    /// * `username` - Username for authentication
    /// * `password` - Password for authentication
    pub fn sasl_plain(username: impl ToString, password: impl ToString) -> Self {
        Self {
            inner: KafkaAuthenticationInner::SaslPlain(wit::KafkaSaslPlainAuth {
                username: username.to_string(),
                password: password.to_string(),
            }),
        }
    }

    /// Creates a SASL/SCRAM SHA256 authentication configuration.
    ///
    /// # Arguments
    ///
    /// * `username` - Username for authentication
    /// * `password` - Password for authentication
    pub fn sasl_scram_sha256(username: impl ToString, password: impl ToString) -> Self {
        Self {
            inner: KafkaAuthenticationInner::SaslScram(wit::KafkaSaslScramAuth {
                username: username.to_string(),
                password: password.to_string(),
                mechanism: wit::KafkaScramMechanism::Sha256,
            }),
        }
    }

    /// Creates a SASL/SCRAM SHA256 authentication configuration.
    ///
    /// # Arguments
    ///
    /// * `username` - Username for authentication
    /// * `password` - Password for authentication
    pub fn sasl_scram_sha512(username: impl ToString, password: impl ToString) -> Self {
        Self {
            inner: KafkaAuthenticationInner::SaslScram(wit::KafkaSaslScramAuth {
                username: username.to_string(),
                password: password.to_string(),
                mechanism: wit::KafkaScramMechanism::Sha512,
            }),
        }
    }

    /// Creates a mTLS authentication configuration.
    ///
    /// # Arguments
    ///
    /// * `cert_path` - Path to the client certificate file
    /// * `key_path` - Path to the client key file
    pub fn mtls(cert_path: impl AsRef<Path>, key_path: impl AsRef<Path>) -> Self {
        Self {
            inner: KafkaAuthenticationInner::Mtls(wit::KafkaMtlsAuth {
                client_cert_path: cert_path.as_ref().to_string_lossy().to_string(),
                client_key_path: key_path.as_ref().to_string_lossy().to_string(),
            }),
        }
    }
}

impl From<KafkaAuthentication> for wit::KafkaAuthentication {
    fn from(value: KafkaAuthentication) -> Self {
        match value.inner {
            KafkaAuthenticationInner::SaslPlain(wit::KafkaSaslPlainAuth { username, password }) => {
                wit::KafkaAuthentication::SaslPlain(wit::KafkaSaslPlainAuth { username, password })
            }
            KafkaAuthenticationInner::SaslScram(wit::KafkaSaslScramAuth {
                username,
                password,
                mechanism,
            }) => wit::KafkaAuthentication::SaslScram(wit::KafkaSaslScramAuth {
                username,
                password,
                mechanism,
            }),
            KafkaAuthenticationInner::Mtls(wit::KafkaMtlsAuth {
                client_cert_path,
                client_key_path,
            }) => wit::KafkaAuthentication::Mtls(wit::KafkaMtlsAuth {
                client_cert_path,
                client_key_path,
            }),
        }
    }
}
