//! Crate error module

/// Error in license signing and verification.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
    /// The provided license is not valid
    #[error("the provided license is invalid")]
    InvalidLicense,
    /// The provided license could not be signed
    #[error("failed to sign the license")]
    SigningFailed,
    /// The provided signing key is not valid
    #[error("the signing key is invalid")]
    InvalidSigningKey,
}
