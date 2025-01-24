#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
#![deny(missing_docs)]

#[doc(hidden)]
pub mod extension;
pub mod host_io;
pub mod types;

pub use extension::{Extension, Resolver};
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

/// Register the extension to the Grafbase Gateway. This macro must be called in the extension
/// crate root for the local extension implementation.
///
/// The first parameter is an [`wit::ExtensionType`], and the second parameter a type implementing
/// the [`extension::Extension`] trait together with the trait matching the extension type.
///
/// For example, to register a resolver extension:
///
/// ```rust
/// struct MyExtension;
///
/// impl grafbase_sdk::Extension for MyExtension {
///     fn new(schema_directives: Vec<grafbase_sdk::types::Directive>) -> Self
///         where Self: Sized
///     {
///         Self
///     }
/// }
///
/// impl grafbase_sdk::Resolver for MyExtension {
///     fn resolve_field(
///         &self,
///         context: grafbase_sdk::SharedContext,
///         directive: grafbase_sdk::types::Directive,
///         inputs: Vec<grafbase_sdk::types::FieldInput>,
///     ) -> Result<grafbase_sdk::types::FieldOutput, grafbase_sdk::Error> {
///         todo!()
///     }
/// }
///
/// grafbase_sdk::register_extension!(grafbase_sdk::ExtensionType::Resolver, MyExtension);
/// ```
#[macro_export]
macro_rules! register_extension {
    ($extension_type:expr, $extension:ty) => {
        #[doc(hidden)]
        #[export_name = "register-extension"]
        pub extern "C" fn __register_extension(host_version: u64) -> i64 {
            let version_result = grafbase_sdk::check_host_version(host_version);

            if version_result < 0 {
                return version_result;
            }

            match $extension_type {
                grafbase_sdk::ExtensionType::Resolver => {
                    // let init_fn = |directives| Box::new(<$extension as grafbase_sdk::Resolver>::new(directives));

                    let init_fn = |directives| {
                        let extension = <$extension as grafbase_sdk::Extension>::new(directives);
                        Box::new(extension) as Box<dyn grafbase_sdk::Resolver>
                    };

                    grafbase_sdk::extension::resolver::register(Box::new(init_fn));
                }
            }

            version_result
        }
    };
}

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
