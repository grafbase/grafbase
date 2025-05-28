use std::time::Duration;

use crate::{SdkError, wit};

use super::{KafkaAuthentication, KafkaTlsConfig};

/// A Kafka producer client for publishing messages to Kafka topics.
pub struct KafkaProducer {
    pub(super) inner: wit::KafkaProducer,
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
            client_config: wit::KafkaClientConfig {
                partitions: value.partitions,
                tls: value.tls,
                authentication: value.authentication,
            },
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
