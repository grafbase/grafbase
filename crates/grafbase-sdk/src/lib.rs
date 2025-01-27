#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
#![deny(missing_docs)]

#[doc(hidden)]
pub mod extension;
pub mod host_io;
pub mod types;

pub use extension::{Extension, Resolver};
pub use grafbase_sdk_derive::ResolverExtension;
pub use wit::{Error, ExtensionType, SharedContext};

use semver::Version;
use std::sync::LazyLock;

/// The minimum version of Grafbase Gateway that can run this SDK.
pub const MINIMUM_HOST_VERSION: Version = Version::new(0, 48, 0);

/// The version of the SDK.
pub static GUEST_VERSION: LazyLock<Version> =
    LazyLock::new(|| Version::parse(std::env!("CARGO_PKG_VERSION")).expect("must be valid semver"));

#[doc(hidden)]
pub fn check_host_version(host_version: u64) -> i64 {
    let version = unpack_version(host_version);

    if version < MINIMUM_HOST_VERSION {
        -(pack_version(&MINIMUM_HOST_VERSION) as i64)
    } else {
        pack_version(&GUEST_VERSION) as i64
    }
}

fn pack_version(version: &Version) -> u64 {
    (version.major << 32) | (version.minor << 16) | version.patch
}

fn unpack_version(version: u64) -> Version {
    Version::new(version >> 32, version >> 16 & 0xFFFF, version & 0xFFFF)
}

struct Component;

#[doc(hidden)]
mod wit {
    #![allow(clippy::too_many_arguments, clippy::missing_safety_doc, missing_docs)]

    wit_bindgen::generate!({
        skip: ["register-extension"],
        path: "./wit/world.wit",
    });
}

wit::export!(Component with_types_in wit);

#[cfg(test)]
mod tests {
    #[test]
    fn test_version_packing() {
        let version = semver::Version::new(1, 2, 3);
        let packed = super::pack_version(&version);
        let unpacked = super::unpack_version(packed);

        assert_eq!(version, unpacked);
    }

    #[test]
    fn test_large_version_packing() {
        let version = semver::Version::new(255, 255, 255);
        let packed = super::pack_version(&version);
        let unpacked = super::unpack_version(packed);

        assert_eq!(version, unpacked);
    }

    #[test]
    fn test_mega_large_version_packing_yolo() {
        let version = semver::Version::new(u16::MAX as u64, u16::MAX as u64, u16::MAX as u64);
        let packed = super::pack_version(&version);
        let unpacked = super::unpack_version(packed);

        assert_eq!(version, unpacked);
    }
}
