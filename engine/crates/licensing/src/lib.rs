//! Provides tools for generating Grafbase license files for the self-hosted gateway.

#![deny(missing_docs)]

mod error;
mod license;

pub use error::Error;
pub use jwt_simple::{
    algorithms::{ECDSAP256KeyPairLike, ECDSAP256PublicKeyLike, ES256KeyPair, ES256PublicKey},
    claims::JWTClaims,
};
pub use license::{in_grace_period, License};

/// The crate result type
pub type Result<T> = std::result::Result<T, Error>;
