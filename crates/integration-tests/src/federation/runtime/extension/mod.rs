mod builder;
mod dispatch;
mod impls;
mod test;

use std::{path::Path, sync::OnceLock};

pub use builder::*;
pub use dispatch::*;
pub use impls::*;
pub use test::*;

const EXTENSIONS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data/extensions/crates");
const PLACEHOLDER_EXTENSION_DIR: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/data/extensions/crates/placeholder/build");

fn placeholder_sdk_version() -> semver::Version {
    static VERSION: OnceLock<semver::Version> = OnceLock::new();
    VERSION
        .get_or_init(|| {
            let Ok(manifest) = std::fs::read_to_string(Path::new(PLACEHOLDER_EXTENSION_DIR).join("manifest.json")) else {
                unreachable!("Failed to read manifest.json for placeholder extension. Please build the integration-tests extensions.");
            };
            let manifest: extension_catalog::VersionedManifest = serde_json::from_str(&manifest).unwrap();
            manifest.into_latest().sdk_version
        })
        .clone()
}
