//! Streaming utilities for gateway-v2
//!
//! Currently this is just re-exports of stuff from gateway_core.
//!
//! At some point we might want to move stuff into this crate and/or a completely
//! separate crate.  But for now this is a good place.

pub use gateway_core::{encode_stream_response, StreamingFormat};
