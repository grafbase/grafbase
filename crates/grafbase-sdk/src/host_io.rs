//! # Host IO modules.
//!
//! This module contains IO modules, providing IO operations to the guest handled by the host runtime.
//! The interfaces are blocking from the guest's perspective, but the host runtime executes the IO asynchronously without
//! blocking the host thread when guest is waiting for IO.

pub mod cache;
pub mod event_queue;
pub mod grpc;
pub mod http;
pub mod kafka;
pub mod logger;
pub mod nats;
pub mod postgres;
