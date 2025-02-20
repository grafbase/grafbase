//! Client interface for interacting with NATS messaging system
//!
//! This module provides a high-level client for connecting to and interacting with NATS servers.
//! It supports both authenticated and unauthenticated connections to one or more NATS servers.

use crate::wit::NatsAuth;

/// A client for interacting with NATS servers
pub struct NatsClient {
    inner: crate::wit::NatsClient,
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
    auth: &NatsAuth,
) -> Result<NatsClient, Box<dyn std::error::Error>> {
    let servers: Vec<_> = servers.into_iter().map(|s| s.to_string()).collect();
    let inner = crate::wit::NatsClient::connect(&servers, Some(auth))?;

    Ok(NatsClient { inner })
}
