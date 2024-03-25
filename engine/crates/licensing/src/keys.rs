//! Definition of licensing keys. In production environment, these keys are distributed
//! through environment variables:
//!
//! The Grafbase gateway must have variable `GRAFBASE_LICENSE_PUBLIC_KEY` during the release build
//! compilation. This variable is used for license verification.
//!
//! The GDN loads the variable `GRAFBASE_LICENSE_PRIVATE_KEY` on runtime when signing a license.
//!
//! Both variables must be base64-encoded.
//!
//! For tests, the keys in the test directory should be used. These keys _must never be the same_
//! as the production keys.

use std::{fs, sync::OnceLock};

use base64::{engine::general_purpose, Engine};

/// The public key for validating license signatures. For release builds, the gateway
/// environment must set this variable to a correct public key.
pub fn public_key() -> &'static [u8] {
    static PUBLIC_KEY: OnceLock<Vec<u8>> = OnceLock::new();

    PUBLIC_KEY.get_or_init(|| match std::option_env!("GRAFBASE_LICENSE_PUBLIC_KEY") {
        Some(key) => general_purpose::STANDARD
            .decode(key)
            .expect("GRAFBASE_LICENSE_PUBLIC_KEY must be base64 encoded"),
        None => fs::read("./test/public-test-key.der").unwrap(),
    })
}

/// The private key for signing licenses.
///
/// ### Panics
///
/// In a production environment, calling this function will panic if GRAFBASE_LICENSE_PRIVATE_KEY
/// is not set.
#[allow(clippy::panic)]
pub fn private_key() -> &'static [u8] {
    static PRIVATE_KEY: OnceLock<Vec<u8>> = OnceLock::new();

    PRIVATE_KEY.get_or_init(|| match std::env::var("GRAFBASE_LICENSE_PRIVATE_KEY") {
        Ok(key) => general_purpose::STANDARD
            .decode(key)
            .expect("GRAFBASE_LICENSE_PRIVATE_KEY must be base64 encoded"),
        Err(_) if cfg!(test) => fs::read("./test/private-test-key.der").unwrap(),
        Err(_) => {
            panic!("GRAFBASE_LICENSE_PRIVATE_KEY must be set in production environments")
        }
    })
}
