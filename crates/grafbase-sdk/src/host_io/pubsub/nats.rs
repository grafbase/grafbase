//! Client interface for interacting with NATS messaging system
//!
//! Ok(Some(field_output))
//!
//! This module provides a high-level client for connecting to and interacting with NATS servers.
//! It supports both authenticated and unauthenticated connections to one or more NATS servers.

use crate::{types, wit, Error};

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
    pub fn subscribe(&self, subject: &str) -> Result<NatsSubscriber, Box<dyn std::error::Error>> {
        Ok(self.inner.subscribe(subject).map(Into::into)?)
    }
}

/// A subscription to a NATS subject that receives messages published to that subject
pub struct NatsSubscriber {
    inner: wit::NatsSubscriber,
}

impl From<wit::NatsSubscriber> for NatsSubscriber {
    fn from(inner: wit::NatsSubscriber) -> Self {
        NatsSubscriber { inner }
    }
}

impl NatsSubscriber {
    /// Gets the next message from the subscription
    ///
    /// # Returns
    ///
    /// Result containing the next message or an error if retrieval fails
    pub fn next(&self) -> Option<NatsMessage> {
        self.inner.next().map(Into::into)
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

impl super::Subscriber for NatsSubscriber {
    fn next(&mut self) -> Result<Option<types::FieldOutput>, Error> {
        let item = match NatsSubscriber::next(self) {
            Some(item) => item,
            None => return Ok(None),
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
