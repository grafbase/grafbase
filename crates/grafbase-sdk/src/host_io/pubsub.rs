//! Subscriber module providing interfaces for consuming field outputs from streams.
//!
//! This module defines the core `Subscriber` trait that abstracts over different
//! implementations of subscribing to field output streams, allowing extensions to
//! handle field outputs in a transport-agnostic way.

pub mod nats;
