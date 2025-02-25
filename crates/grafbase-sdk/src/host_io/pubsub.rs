//! Subscriber module providing interfaces for consuming field outputs from streams.
//!
//! This module defines the core `Subscriber` trait that abstracts over different
//! implementations of subscribing to field output streams, allowing extensions to
//! handle field outputs in a transport-agnostic way.

use crate::{types::FieldOutput, Error};

pub mod nats;

/// A trait for consuming field outputs from streams.
///
/// This trait provides an abstraction over different implementations
/// of subscriptions to field output streams. Implementors should handle
/// the details of their specific transport mechanism while providing a
/// consistent interface for consumers.
pub trait Subscription {
    /// Retrieves the next field output from the subscription.
    ///
    /// Returns:
    /// - `Ok(Some(FieldOutput))` if a field output was available
    /// - `Ok(None)` if the subscription has ended normally
    /// - `Err(Error)` if an error occurred while retrieving the next field output
    fn next(&mut self) -> Result<Option<FieldOutput>, Error>;
}
