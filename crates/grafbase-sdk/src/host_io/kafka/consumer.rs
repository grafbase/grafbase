use std::{borrow::Cow, time::Duration};

use chrono::{DateTime, Utc};

use crate::{
    SdkError, Subscription,
    types::{Error, Response, SubscriptionItem},
    wit,
};

use super::{KafkaAuthentication, KafkaTlsConfig};

/// A Kafka consumer that can receive messages from Kafka topics.
pub struct KafkaConsumer {
    pub(super) inner: wit::KafkaConsumer,
}

impl KafkaConsumer {
    /// Retrieves the next message from the Kafka consumer.
    ///
    /// Returns `Ok(Some(message))` if a message is available,
    /// `Ok(None)` if no message is currently available,
    /// or `Err` if an error occurred while consuming.
    pub fn next(&self) -> Result<Option<KafkaMessage>, SdkError> {
        self.inner.next().map_err(Into::into).map(|msg| msg.map(Into::into))
    }
}

impl Subscription for KafkaConsumer {
    fn next(&mut self) -> Result<Option<SubscriptionItem>, Error> {
        match KafkaConsumer::next(self) {
            Ok(Some(msg)) => Ok(Some(
                Response::json(msg.inner.value.unwrap_or_else(|| b"null".into())).into(),
            )),
            Ok(None) => Ok(None),
            Err(err) => Err(format!("Error receiving Kafka message: {err}").into()),
        }
    }
}

/// A Kafka message containing key, value, headers, and metadata.
pub struct KafkaMessage {
    inner: wit::KafkaMessage,
}

impl KafkaMessage {
    /// Returns the message key as a UTF-8 string, if present.
    ///
    /// The key is used for partitioning and message ordering within a partition.
    pub fn key(&self) -> Option<Cow<'_, str>> {
        self.raw_key().map(|key| String::from_utf8_lossy(key))
    }

    /// Returns the raw message key as bytes, if present.
    ///
    /// This provides access to the unprocessed message key without UTF-8 conversion.
    pub fn raw_key(&self) -> Option<&[u8]> {
        self.inner.key.as_deref()
    }

    /// Deserializes the message value from JSON.
    pub fn value<S>(&self) -> Result<Option<S>, SdkError>
    where
        S: for<'de> serde::Deserialize<'de>,
    {
        match self.raw_value() {
            Some(value) => serde_json::from_slice(value).map_err(Into::into),
            None => Ok(None),
        }
    }

    /// Returns the raw message value as bytes, if present.
    ///
    /// This provides access to the unprocessed message value without any JSON deserialization.
    pub fn raw_value(&self) -> Option<&[u8]> {
        self.inner.value.as_deref()
    }

    /// Consumes the message and returns the raw value as a `Vec<u8>`, if present.
    pub fn into_raw_value(self) -> Option<Vec<u8>> {
        self.inner.value
    }

    /// Returns the message offset within the partition.
    ///
    /// The offset is a unique identifier for the message within its partition.
    pub fn offset(&self) -> i64 {
        self.inner.offset
    }

    /// Gets a header value by key and deserializes it from JSON.
    ///
    /// Returns `Ok(Some(value))` if the header exists and can be deserialized,
    /// `Ok(None)` if the header doesn't exist,
    /// or `Err` if deserialization fails.
    pub fn get_header_value<S>(&self, key: &str) -> Result<Option<S>, SdkError>
    where
        S: for<'de> serde::Deserialize<'de>,
    {
        match self.get_raw_header_value(key) {
            Some(value) => {
                let value = serde_json::from_slice(value)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Gets a raw header value by key as bytes.
    pub fn get_raw_header_value(&self, key: &str) -> Option<&[u8]> {
        // The kafka headers come as btree, which means they are sorted by the key.
        // This means we can use binary search to find the header value.
        match self.inner.headers.binary_search_by(|item| item.0.as_str().cmp(key)) {
            Ok(index) => Some(self.inner.headers[index].1.as_ref()),
            Err(_) => None,
        }
    }

    /// Returns the timestamp when the message was produced.
    ///
    /// The timestamp is in UTC and represents when the message was created.
    pub fn timestamp(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.inner.timestamp, 0)
            .expect("we converted this from a datetime in the host, it must be valid")
    }

    /// Returns the high watermark for the partition.
    ///
    /// The high watermark indicates the offset of the last message + 1 in the partition.
    pub fn high_watermark(&self) -> i64 {
        self.inner.high_watermark
    }
}

impl From<wit::KafkaMessage> for KafkaMessage {
    fn from(inner: wit::KafkaMessage) -> Self {
        Self { inner }
    }
}

/// Configuration for a Kafka consumer.
///
/// This struct contains settings that control how the consumer behaves,
/// including batch sizes, wait times, client configuration, and starting offset.
pub struct KafkaConsumerConfig {
    /// Minimum number of messages to batch together before returning.
    min_batch_size: Option<i32>,
    /// Maximum number of messages to batch together.
    max_batch_size: Option<i32>,
    /// Maximum time to wait for messages before returning a partial batch.
    max_wait_time: Option<Duration>,
    /// Client configuration including partitions, TLS, and authentication.
    client_config: wit::KafkaClientConfig,
    /// The offset position to start consuming from.
    start_offset: wit::KafkaConsumerStartOffset,
}

impl KafkaConsumerConfig {
    /// Sets the minimum number of messages to batch together before returning.
    pub fn min_batch_size(&mut self, min_batch_size: i32) {
        self.min_batch_size = Some(min_batch_size);
    }

    /// Sets the maximum number of messages to batch together.
    pub fn max_batch_size(&mut self, max_batch_size: i32) {
        self.max_batch_size = Some(max_batch_size);
    }

    /// Sets the maximum time to wait for messages before returning a partial batch.
    pub fn max_wait_time(&mut self, max_wait_time: Duration) {
        self.max_wait_time = Some(max_wait_time);
    }

    /// Configures the consumer to start consuming from the latest offset.
    ///
    /// This means the consumer will only receive new messages that arrive after it starts.
    pub fn from_latest_offset(&mut self) {
        self.start_offset = wit::KafkaConsumerStartOffset::Latest;
    }

    /// Configures the consumer to start consuming from the earliest offset.
    ///
    /// This means the consumer will receive all available messages in the topic from the beginning.
    pub fn from_earliest_offset(&mut self) {
        self.start_offset = wit::KafkaConsumerStartOffset::Earliest;
    }

    /// Configures the consumer to start consuming from a specific offset.
    pub fn from_specific_offset(&mut self, offset: i64) {
        self.start_offset = wit::KafkaConsumerStartOffset::Specific(offset);
    }

    /// Sets the TLS configuration for secure connections to Kafka brokers.
    pub fn tls(&mut self, tls: KafkaTlsConfig) {
        self.client_config.tls = Some(tls.into());
    }

    /// Sets the authentication configuration for connecting to Kafka brokers.
    pub fn authentication(&mut self, authentication: KafkaAuthentication) {
        self.client_config.authentication = Some(authentication.into());
    }

    /// Sets the specific partitions to consume from.
    ///
    /// If not specified, the consumer will consume from all partitions of the topic.
    pub fn partitions(&mut self, partitions: Vec<i32>) {
        self.client_config.partitions = Some(partitions);
    }
}

impl Default for KafkaConsumerConfig {
    fn default() -> Self {
        Self {
            min_batch_size: None,
            max_batch_size: None,
            max_wait_time: None,
            client_config: wit::KafkaClientConfig {
                partitions: None,
                tls: None,
                authentication: None,
            },
            start_offset: wit::KafkaConsumerStartOffset::Latest,
        }
    }
}

impl From<KafkaConsumerConfig> for wit::KafkaConsumerConfig {
    fn from(value: KafkaConsumerConfig) -> Self {
        Self {
            min_batch_size: value.min_batch_size,
            max_batch_size: value.max_batch_size,
            max_wait_ms: value.max_wait_time.map(|ms| ms.as_millis() as i32),
            client_config: value.client_config,
            start_offset: value.start_offset,
        }
    }
}
