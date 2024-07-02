use core::fmt;

use wasmtime::component::{ComponentType, Lift};

/// An error type available for the user to throw from the guest.
#[derive(Clone, ComponentType, Lift, Debug, thiserror::Error, PartialEq)]
#[component(record)]
pub struct Error {
    /// Additional extensions added to the GraphQL response
    pub extensions: Vec<(String, String)>,
    /// The error message
    pub message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}
