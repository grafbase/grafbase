use std::{
    collections::{HashMap, btree_map::Entry},
    path::{Path, PathBuf},
    sync::Arc,
};

use extension_catalog::{Extension, ExtensionCatalog, ExtensionId, Manifest};
use url::Url;
use wasi_component_loader::{extension::WasmExtensions, resources::SharedResources};

use crate::gateway::DispatchRule;

use super::{
    EXTENSIONS_DIR, ExtensionsDispatcher, PLACEHOLDER_EXTENSION_DIR, TestExtensions, TestExtensionsState, TestManifest,
    placeholder_sdk_version,
};

pub struct ExtensionsBuilder {
    tmpdir: PathBuf,
    catalog: ExtensionCatalog,
    has_wasm_extension: bool,
    dispatch: HashMap<ExtensionId, DispatchRule>,
    pub test: TestExtensionsState,
}

pub trait AnyExtension {
    fn register(self, state: &mut ExtensionsBuilder);
}

impl AnyExtension for &'static str {
    fn register(self, state: &mut ExtensionsBuilder) {
        state.push_wasm_extension(self, Path::new(EXTENSIONS_DIR).join(self).join("build"))
    }
}

impl ExtensionsBuilder {
    pub fn new(tmpdir: PathBuf) -> Self {
        Self {
            tmpdir,
            catalog: ExtensionCatalog::default(),
            has_wasm_extension: false,
            dispatch: HashMap::new(),
            test: TestExtensionsState::default(),
        }
    }

    pub fn get_url(&self, id: &str) -> url::Url {
        let Some((extension_id, _)) = self
            .catalog
            .iter_with_id()
            .find(|(_, ext)| ext.manifest.id.to_string() == id)
        else {
            panic!("Extension '{}' not found", id);
        };

        self.url(extension_id)
    }

    fn url(&self, extension_id: ExtensionId) -> Url {
        let extension = &self.catalog[extension_id];
        match self.dispatch[&extension_id] {
            DispatchRule::Wasm => Url::from_file_path(extension.wasm_path.parent().unwrap()).unwrap(),
            DispatchRule::Test => Url::from_file_path(self.tmpdir.join(extension.manifest.id.to_string())).unwrap(),
        }
    }

    #[track_caller]
    pub fn push_wasm_extension(&mut self, name: &'static str, dir: PathBuf) {
        self.has_wasm_extension = true;
        let manifest_path = dir.join("manifest.json");
        let wasm_path = dir.join("extension.wasm");
        if !dir.is_dir() || !manifest_path.is_file() || !wasm_path.is_file() {
            panic!("Extension '{}' not found in {}", name, dir.display());
        }
        // Yeah it's profoundly ugly but does provide a nicer consistent top-level API for tests.
        let Ok(manifest) = std::fs::read_to_string(manifest_path) else {
            unreachable!(
                "Failed to read manifest.json for placeholder extension. Please build the integration-tests extensions."
            );
        };
        let manifest: extension_catalog::VersionedManifest = serde_json::from_str(&manifest).unwrap();
        let extension = Extension {
            manifest: manifest.into_latest(),
            wasm_path,
        };
        let id = self.catalog.push(extension);
        self.dispatch.insert(id, DispatchRule::Wasm);
    }

    #[track_caller]
    pub fn push_test_extension(&mut self, manifest: TestManifest) -> ExtensionId {
        let TestManifest { id, sdl, r#type } = manifest;

        let manifest = extension_catalog::Manifest {
            id,
            r#type,
            sdk_version: placeholder_sdk_version(),
            minimum_gateway_version: "0.0.0".parse().unwrap(),
            sdl: sdl.map(str::to_string),
            description: "Test extension".to_owned(),
            homepage_url: None,
            license: None,
            readme: None,
            repository_url: None,
            permissions: Default::default(),
        };

        let dir = self.tmpdir.join(manifest.id.to_string());
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("manifest.json"),
            serde_json::to_vec(&manifest.clone().into_versioned()).unwrap(),
        )
        .unwrap();
        let id = self.catalog.push(Extension {
            manifest,
            wasm_path: Path::new(PLACEHOLDER_EXTENSION_DIR).join("extension.wasm"),
        });
        self.dispatch.insert(id, DispatchRule::Test);
        id
    }

    pub fn catalog(&self) -> &ExtensionCatalog {
        &self.catalog
    }

    pub fn iter_with_url(&self) -> impl Iterator<Item = (&Manifest, Url)> {
        self.catalog
            .iter_with_id()
            .map(move |(id, ext)| (&ext.manifest, self.url(id)))
    }

    pub async fn build_and_ingest_catalog_into_config(
        self,
        config: &mut gateway_config::Config,
        schema: &Arc<engine::Schema>,
        shared_resources: SharedResources,
    ) -> Result<(ExtensionsDispatcher, ExtensionCatalog), String> {
        let wasm_extensions = if self.has_wasm_extension {
            for ext in self.catalog.iter() {
                let version = ext.manifest.id.version.to_string().parse().unwrap();
                let path = Some(ext.wasm_path.parent().unwrap().to_path_buf());
                match config.extensions.entry(ext.manifest.name().to_string()) {
                    Entry::Vacant(entry) => {
                        entry.insert(gateway_config::ExtensionConfig::Structured(
                            gateway_config::StructuredExtensionConfig {
                                version,
                                path,
                                ..Default::default()
                            },
                        ));
                    }
                    Entry::Occupied(mut entry) => {
                        let value = entry.get_mut();
                        match value {
                            gateway_config::ExtensionConfig::Structured(config) => {
                                config.version = version;
                                config.path = path;
                            }
                            gateway_config::ExtensionConfig::Version(_) => {
                                return Err("Extension with the same name already exists".to_owned());
                            }
                        }
                    }
                }
            }

            WasmExtensions::new(shared_resources, &self.catalog, config, schema)
                .await
                .map_err(|err| err.to_string())?
        } else {
            // If no real wasm extensions was used, we skip the initialization as it would compile
            // the placeholder extension for nothing and we have a lot of extension tests, most of
            // them not relying on wasm at all.
            Default::default()
        };

        let extensions = ExtensionsDispatcher {
            wasm: wasm_extensions,
            test: TestExtensions {
                state: Arc::new(tokio::sync::Mutex::new(self.test)),
            },
            dispatch: self.dispatch,
        };

        Ok((extensions, self.catalog))
    }
}
