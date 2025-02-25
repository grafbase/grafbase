//! Client interface for interacting with NATS messaging system
//!
//! Ok(Some(field_output))
//!
//! This module provides a high-level client for connecting to and interacting with NATS servers.
//! It supports both authenticated and unauthenticated connections to one or more NATS servers.

use crate::{extension::resolver::Subscription, types, wit, Error};
use std::time::Duration;

pub use time::OffsetDateTime;

/// A client for interacting with NATS servers
pub struct NatsClient {
    inner: wit::NatsClient,
}

impl NatsClient {
    /// Publishes a message to the specified NATS subject
    ///
    /// # Arguments
    ///
    /// * `subject` - The NATS subject to publish to
    /// * `payload` - The message payload as a byte slice
    ///
    /// # Returns
    ///
    /// Result indicating success or an error if the publish fails
    pub fn publish<S>(&self, subject: &str, payload: &S) -> Result<(), Box<dyn std::error::Error>>
    where
        S: serde::Serialize,
    {
        Ok(self.inner.publish(subject, &serde_json::to_vec(payload).unwrap())?)
    }

    /// Subscribes to messages on the specified NATS subject
    ///
    /// # Arguments
    ///
    /// * `subject` - The NATS subject to subscribe to
    ///
    /// # Returns
    ///
    /// Result containing the subscription or an error if subscription fails
    pub fn subscribe(
        &self,
        subject: &str,
        config: Option<NatsStreamConfig>,
    ) -> Result<NatsSubscription, Box<dyn std::error::Error>> {
        let subscription = self
            .inner
            .subscribe(subject, config.map(Into::into).as_ref())
            .map(Into::into)?;

        Ok(subscription)
    }
}

/// A subscription to a NATS subject that receives messages published to that subject
pub struct NatsSubscription {
    inner: wit::NatsSubscriber,
}

impl From<wit::NatsSubscriber> for NatsSubscription {
    fn from(inner: wit::NatsSubscriber) -> Self {
        NatsSubscription { inner }
    }
}

impl NatsSubscription {
    /// Gets the next message from the subscription
    ///
    /// # Returns
    ///
    /// Result containing the next message or an error if retrieval fails
    pub fn next(&self) -> Result<Option<NatsMessage>, Box<dyn std::error::Error>> {
        Ok(self.inner.next()?.map(Into::into))
    }
}

/// A message received from a NATS subscription containing the payload data
pub struct NatsMessage {
    inner: crate::wit::NatsMessage,
}

impl From<crate::wit::NatsMessage> for NatsMessage {
    fn from(inner: crate::wit::NatsMessage) -> Self {
        NatsMessage { inner }
    }
}

impl NatsMessage {
    /// Gets the payload data of the message
    ///
    /// # Returns
    ///
    /// Result containing the payload data or an error if retrieval fails
    pub fn payload<S>(&self) -> anyhow::Result<S>
    where
        S: for<'de> serde::Deserialize<'de>,
    {
        Ok(serde_json::from_slice(&self.inner.payload)?)
    }

    /// Gets the subject of the message
    ///
    /// # Returns
    ///
    /// The NATS subject this message was published to
    pub fn subject(&self) -> &str {
        &self.inner.subject
    }
}

/// Connects to one or more NATS servers
///
/// # Arguments
///
/// * `servers` - Iterator of server addresses to connect to
///
/// # Returns
///
/// Result containing the connected NATS client or an error if connection fails
pub fn connect(servers: impl IntoIterator<Item = impl ToString>) -> Result<NatsClient, Box<dyn std::error::Error>> {
    let servers: Vec<_> = servers.into_iter().map(|s| s.to_string()).collect();
    let inner = crate::wit::NatsClient::connect(&servers, None)?;

    Ok(NatsClient { inner })
}

/// Connects to one or more NATS servers with authentication
///
/// # Arguments
///
/// * `servers` - Iterator of server addresses to connect to
/// * `auth` - Authentication credentials for connecting to the servers
///
/// # Returns
///
/// Result containing the connected NATS client or an error if connection fails
pub fn connect_with_auth(
    servers: impl IntoIterator<Item = impl ToString>,
    auth: &crate::NatsAuth,
) -> Result<NatsClient, Box<dyn std::error::Error>> {
    let servers: Vec<_> = servers.into_iter().map(|s| s.to_string()).collect();
    let inner = crate::wit::NatsClient::connect(&servers, Some(auth))?;

    Ok(NatsClient { inner })
}

impl Subscription for NatsSubscription {
    fn next(&mut self) -> Result<Option<types::FieldOutput>, Error> {
        let item = match NatsSubscription::next(self) {
            Ok(Some(item)) => item,
            Ok(None) => return Ok(None),
            Err(e) => {
                return Err(Error {
                    extensions: Vec::new(),
                    message: format!("Error receiving NATS message: {e}"),
                })
            }
        };

        let mut field_output = types::FieldOutput::default();

        let payload: serde_json::Value = item.payload().map_err(|e| Error {
            extensions: Vec::new(),
            message: format!("Error parsing NATS value as JSON: {e}"),
        })?;

        field_output.push_value(payload);

        Ok(Some(field_output))
    }
}

/// Configuration for NATS JetStream consumers
///
/// This struct wraps the internal configuration for JetStream consumers
/// and provides a builder pattern for easy configuration.
pub struct NatsStreamConfig(wit::NatsStreamConfig);

impl From<NatsStreamConfig> for wit::NatsStreamConfig {
    fn from(config: NatsStreamConfig) -> Self {
        config.0
    }
}

/// Delivery policy for NATS JetStream consumers
///
/// This enum defines the various policies that determine how messages are delivered to
/// JetStream consumers, such as delivering all messages, only the latest message,
/// or messages from a specific sequence or time.
#[derive(Debug)]
pub enum NatsStreamDeliverPolicy {
    /// All causes the consumer to receive the oldest messages still present in the system.
    /// This is the default.
    All,
    /// Last will start the consumer with the last sequence received.
    Last,
    /// New will only deliver new messages that are received by the JetStream server after
    /// the consumer is created.
    New,
    /// ByStartSequence will look for a defined starting sequence to the consumer’s configured
    /// opt_start_seq parameter.
    ByStartSequence(u64),
    /// ByStartTime will select the first message with a timestamp >= to the consumer’s
    /// configured opt_start_time parameter.
    ByStartTime(OffsetDateTime),
    /// LastPerSubject will start the consumer with the last message for all subjects received.
    LastPerSubject,
}

impl From<NatsStreamDeliverPolicy> for wit::NatsStreamDeliverPolicy {
    fn from(value: NatsStreamDeliverPolicy) -> Self {
        match value {
            NatsStreamDeliverPolicy::All => wit::NatsStreamDeliverPolicy::All,
            NatsStreamDeliverPolicy::Last => wit::NatsStreamDeliverPolicy::Last,
            NatsStreamDeliverPolicy::New => wit::NatsStreamDeliverPolicy::New,
            NatsStreamDeliverPolicy::ByStartSequence(seq) => wit::NatsStreamDeliverPolicy::ByStartSequence(seq),
            NatsStreamDeliverPolicy::ByStartTime(time) => {
                wit::NatsStreamDeliverPolicy::ByStartTimeMs((time.unix_timestamp_nanos() / 1_000_000) as u64)
            }
            NatsStreamDeliverPolicy::LastPerSubject => wit::NatsStreamDeliverPolicy::LastPerSubject,
        }
    }
}

impl NatsStreamConfig {
    /// Creates a new JetStream consumer configuration
    ///
    /// # Arguments
    ///
    /// * `deliver_policy` - Determines how messages are delivered to the consumer
    /// * `inactive_threshold` - Duration after which a consumer is considered inactive
    ///
    /// # Returns
    ///
    /// A new `NatsStreamConfig` with the specified settings
    pub fn new(
        stream_name: String,
        consumer_name: String,
        deliver_policy: NatsStreamDeliverPolicy,
        inactive_threshold: Duration,
    ) -> Self {
        NatsStreamConfig(wit::NatsStreamConfig {
            stream_name,
            consumer_name,
            durable_name: None,
            deliver_policy: deliver_policy.into(),
            inactive_threshold_ms: inactive_threshold.as_millis() as u64,
            description: None,
        })
    }

    /// Sets a durable name for the consumer
    ///
    /// Durable consumers maintain their state even when disconnected.
    ///
    /// # Arguments
    ///
    /// * `durable_name` - The durable name to use for this consumer
    ///
    /// # Returns
    ///
    /// The updated configuration
    pub fn with_durable_name(mut self, durable_name: String) -> Self {
        self.0.durable_name = Some(durable_name);
        self
    }

    /// Sets a description for the consumer
    ///
    /// # Arguments
    ///
    /// * `description` - The description to use for this consumer
    ///
    /// # Returns
    ///
    /// The updated configuration
    pub fn with_description(mut self, description: String) -> Self {
        self.0.description = Some(description);
        self
    }
}
