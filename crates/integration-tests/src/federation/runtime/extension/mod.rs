mod builder;
mod dispatch;
mod test;

use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

pub use builder::*;
pub use dispatch::*;
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

pub enum WasmOrTestExtension {
    Wasm(WasmExtension),
    Test(Box<dyn TestExtensionBuilder>),
}

impl From<&'static str> for WasmOrTestExtension {
    fn from(name: &'static str) -> Self {
        WasmOrTestExtension::Wasm(WasmExtension {
            name: name.to_string(),
            dir: Path::new(EXTENSIONS_DIR).join(name).join("build"),
        })
    }
}

impl<B: TestExtensionBuilder + Sized> From<B> for WasmOrTestExtension {
    fn from(builder: B) -> Self {
        WasmOrTestExtension::Test(Box::new(builder))
    }
}

pub struct WasmExtension {
    pub name: String,
    pub dir: PathBuf,
}
