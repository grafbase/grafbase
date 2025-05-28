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

use crate::{SdkError, wit};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

/// Connects to Kafka servers and creates a new Kafka producer.
///
/// # Arguments
///
/// * `name` - The name identifier for this producer
/// * `servers` - An iterable of server addresses to connect to
/// * `topic` - The Kafka topic to produce messages to
/// * `config` - Configuration settings for the producer
pub fn producer(
    name: &str,
    servers: impl IntoIterator<Item = impl ToString>,
    topic: &str,
    config: KafkaProducerConfig,
) -> Result<KafkaProducer, SdkError> {
    let servers: Vec<_> = servers.into_iter().map(|s| s.to_string()).collect();
    let config = config.into();
    let producer = wit::KafkaProducer::connect(name, &servers, topic, Some(&config))?;

    Ok(KafkaProducer { inner: producer })
}

/// A Kafka producer client for publishing messages to Kafka topics.
pub struct KafkaProducer {
    inner: wit::KafkaProducer,
}

impl KafkaProducer {
    /// Produces a message to the Kafka topic.
    ///
    /// # Arguments
    ///
    /// * `key` - Optional message key for partitioning and message ordering
    /// * `value` - The message payload as bytes
    pub fn produce(&self, key: Option<&str>, value: &[u8]) -> Result<(), SdkError> {
        self.inner.produce(key, value)?;
        Ok(())
    }
}

/// Configuration options for a Kafka producer.
pub struct KafkaProducerConfig {
    /// Compression algorithm to use for message batches
    compression: wit::KafkaProducerCompression,
    /// Optional list of specific partitions to produce to
    partitions: Option<Vec<i32>>,
    /// Optional batching configuration for message processing
    batching: Option<KafkaBatchConfig>,
    /// Optional TLS configuration for secure connections
    tls: Option<wit::KafkaTlsConfig>,
    /// Optional authentication configuration
    authentication: Option<wit::KafkaAuthentication>,
}

impl KafkaProducerConfig {
    /// Sets the compression algorithm to use for message batches.
    ///
    /// # Arguments
    ///
    /// * `compression` - The compression algorithm to apply to message batches
    pub fn compression(&mut self, compression: KafkaProducerCompression) {
        self.compression = compression.into();
    }

    /// Sets the specific partitions that this producer should send messages to.
    ///
    /// # Arguments
    ///
    /// * `partitions` - A list of partition IDs to produce messages to
    pub fn partitions(&mut self, partitions: Vec<i32>) {
        self.partitions = Some(partitions);
    }

    /// Sets the batching configuration for message processing.
    ///
    /// # Arguments
    ///
    /// * `batching` - The batching configuration settings to use
    pub fn batching(&mut self, batching: KafkaBatchConfig) {
        self.batching = Some(batching);
    }

    /// Sets the TLS configuration for secure connections to Kafka brokers.
    pub fn tls(&mut self, tls: KafkaTlsConfig) {
        self.tls = Some(tls.into());
    }

    /// Sets the authentication configuration for connecting to Kafka brokers.
    ///
    /// # Arguments
    ///
    /// * `authentication` - The authentication settings to use
    pub fn authentication(&mut self, authentication: KafkaAuthentication) {
        self.authentication = Some(authentication.into());
    }
}

/// Configuration options for Kafka message batching.
pub struct KafkaBatchConfig {
    /// Maximum number of bytes to include in a single batch
    pub max_size_bytes: u64,
    /// Time to wait before sending a batch of messages
    pub linger: Duration,
}

/// Compression algorithms available for Kafka message batches.
pub enum KafkaProducerCompression {
    /// No compression applied to messages
    None,
    /// Gzip compression algorithm
    Gzip,
    /// Snappy compression algorithm
    Snappy,
    /// LZ4 compression algorithm
    Lz4,
    /// Zstandard compression algorithm
    Zstd,
}

impl From<KafkaProducerCompression> for wit::KafkaProducerCompression {
    fn from(value: KafkaProducerCompression) -> Self {
        match value {
            KafkaProducerCompression::None => wit::KafkaProducerCompression::None,
            KafkaProducerCompression::Gzip => wit::KafkaProducerCompression::Gzip,
            KafkaProducerCompression::Snappy => wit::KafkaProducerCompression::Snappy,
            KafkaProducerCompression::Lz4 => wit::KafkaProducerCompression::Lz4,
            KafkaProducerCompression::Zstd => wit::KafkaProducerCompression::Zstd,
        }
    }
}

impl From<KafkaProducerConfig> for wit::KafkaProducerConfig {
    fn from(value: KafkaProducerConfig) -> Self {
        Self {
            compression: value.compression,
            partitions: value.partitions,
            tls: value.tls,
            authentication: value.authentication,
            batching: value.batching.map(Into::into),
        }
    }
}

impl From<KafkaBatchConfig> for wit::KafkaBatchConfig {
    fn from(value: KafkaBatchConfig) -> Self {
        Self {
            linger_ms: value.linger.as_millis() as u64,
            batch_size_bytes: value.max_size_bytes,
        }
    }
}

impl Default for KafkaProducerConfig {
    fn default() -> Self {
        Self {
            compression: wit::KafkaProducerCompression::None,
            partitions: None,
            batching: None,
            tls: None,
            authentication: None,
        }
    }
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
