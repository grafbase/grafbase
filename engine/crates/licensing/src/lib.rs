//! Provides tools for generating Grafbase license files for the self-hosted gateway.

#![deny(missing_docs)]

mod error;
pub mod keys;
mod license;

pub use error::Error;
pub use license::{License, SignedLicense, SIGNING_ALGORITHM, VERIFICATION_ALGORITHM};

/// The crate result type
pub type Result<T> = std::result::Result<T, Error>;
