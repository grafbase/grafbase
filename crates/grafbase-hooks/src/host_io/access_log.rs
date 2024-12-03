//! Provides a simple access log for the gateway.
//!
//! The access logs must be enabled in the gateway configuration for this module to work:
//!
//! ```toml,no_run
//! [gateway.access_logs]
//! enabled = true
//! path = "/path/to/logs"
//! rotate = "daily"
//! mode = "blocking"
//! ```

pub use crate::wit::{AccessLog, LogError};

/// Stores the given arbitrary bytes to the access log. The data can be generated in any format,
/// and must be serialized as bytes before sending.
pub fn send(data: &[u8]) -> Result<(), LogError> {
    AccessLog::send(data)
}
