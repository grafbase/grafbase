//! Subscriber module providing interfaces for consuming field outputs from streams.
//!
//! This module defines the core `Subscriber` trait that abstracts over different
//! implementations of subscribing to field output streams, allowing extensions to
//! handle field outputs in a transport-agnostic way.

use crate::{types::FieldOutput, Error};

pub mod nats;

/// A trait for subscribing to a stream of field outputs.
pub trait Subscriber {
    /// Gets the next field output from the stream.
    ///
    /// Returns `Ok(Some(output))` if a field output is available,
    /// `Ok(None)` if the stream has ended, or `Err` if an error occurred.
    fn next(&mut self) -> Result<Option<FieldOutput>, Error>;
}
