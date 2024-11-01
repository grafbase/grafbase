use core::fmt;

use wasmtime::component::{ComponentType, Lift};

/// An error type which defines a full HTTP response.
#[derive(Clone, ComponentType, Lift, Debug, PartialEq, thiserror::Error)]
#[component(record)]
pub struct ErrorResponse {
    #[component(name = "status-code")]
    pub status_code: u16,
    pub errors: Vec<GuestError>,
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HTTP {}", self.status_code)
    }
}

/// An error type available for the user to throw from the guest.
#[derive(Clone, ComponentType, Lift, Debug, thiserror::Error, PartialEq)]
#[component(record)]
pub struct GuestError {
    /// Additional extensions added to the GraphQL response
    pub extensions: Vec<(String, String)>,
    /// The error message
    pub message: String,
}

impl fmt::Display for GuestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}
