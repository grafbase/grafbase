use std::{
    collections::{HashMap, btree_map::Entry},
    path::Path,
    sync::Arc,
};

use engine_schema::SubgraphId;
use extension_catalog::{Extension, ExtensionCatalog, ExtensionId, Manifest};
use tokio::sync::Mutex;
use url::Url;
use wasi_component_loader::{create_access_log_channel, extension::WasmExtensions, resources::SharedResources};

use crate::federation::DispatchRule;

use super::{
    ExtensionsDispatcher, PLACEHOLDER_EXTENSION_DIR, TestExtension, TestExtensionBuilder, TestExtensions, TestManifest,
    WasmExtension, placeholder_sdk_version,
};

pub struct ExtensionsBuilder {
    tmpdir: tempfile::TempDir,
    catalog: ExtensionCatalog,
    has_wasm_extension: bool,
    dispatch: HashMap<ExtensionId, DispatchRule>,
    builders: HashMap<ExtensionId, Box<dyn TestExtensionBuilder>>,
    global_instances: Mutex<HashMap<ExtensionId, Arc<dyn TestExtension>>>,
    subgraph_instances: Mutex<HashMap<(ExtensionId, SubgraphId), Arc<dyn TestExtension>>>,
}

impl Default for ExtensionsBuilder {
    fn default() -> Self {
        Self {
            tmpdir: tempfile::tempdir().unwrap(),
            catalog: Default::default(),
            has_wasm_extension: false,
            dispatch: Default::default(),
            builders: Default::default(),
            global_instances: Default::default(),
            subgraph_instances: Default::default(),
        }
    }
}

impl ExtensionsBuilder {
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
            DispatchRule::Test => {
                Url::from_file_path(self.tmpdir.path().join(extension.manifest.id.to_string())).unwrap()
            }
        }
    }

    #[track_caller]
    pub fn push_wasm_extension(&mut self, ext: WasmExtension) {
        self.has_wasm_extension = true;
        let manifest_path = ext.dir.join("manifest.json");
        let wasm_path = ext.dir.join("extension.wasm");
        if !ext.dir.is_dir() || !manifest_path.is_file() || !wasm_path.is_file() {
            panic!("Extension '{}' not found in {}", ext.name, ext.dir.display());
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
    pub fn push_test_extension(&mut self, builder: Box<dyn TestExtensionBuilder>) {
        let TestManifest { id, sdl, kind } = builder.manifest();

        let manifest = extension_catalog::Manifest {
            id,
            kind,
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

        let dir = self.tmpdir.path().join(manifest.id.to_string());
        std::fs::create_dir(&dir).unwrap();
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
        self.builders.insert(id, builder);
    }

    pub fn catalog(&self) -> &ExtensionCatalog {
        &self.catalog
    }

    pub fn iter_with_url(&self) -> impl Iterator<Item = (&Manifest, Url)> {
        self.catalog
            .iter_with_id()
            .map(move |(id, ext)| (&ext.manifest, self.url(id)))
    }

    pub async fn build(
        self,
        mut config: gateway_config::Config,
        schema: &engine::Schema,
    ) -> Result<ExtensionsDispatcher, String> {
        let meter = grafbase_telemetry::metrics::meter_from_global_provider();
        let counter = meter.i64_up_down_counter("grafbase.gateway.access_log.pending").build();
        let (access_log_sender, access_log_receiver) = create_access_log_channel(false, counter);

        let wasm_extensions = if self.has_wasm_extension {
            for ext in self.catalog.iter() {
                let version = ext.manifest.id.version.to_string().parse().unwrap();
                let path = Some(ext.wasm_path.parent().unwrap().to_path_buf());
                match config.extensions.entry(ext.manifest.name().to_string()) {
                    Entry::Vacant(entry) => {
                        entry.insert(gateway_config::ExtensionsConfig::Structured(
                            gateway_config::StructuredExtensionsConfig {
                                version,
                                path,
                                ..Default::default()
                            },
                        ));
                    }
                    Entry::Occupied(mut entry) => {
                        let value = entry.get_mut();
                        match value {
                            gateway_config::ExtensionsConfig::Structured(config) => {
                                config.version = version;
                                config.path = path;
                            }
                            gateway_config::ExtensionsConfig::Version(_) => {
                                return Err("Extension with the same name already exists".to_owned());
                            }
                        }
                    }
                }
            }

            WasmExtensions::new(
                SharedResources {
                    access_log: access_log_sender,
                },
                &self.catalog,
                &config,
                schema,
            )
            .await
            .unwrap()
        } else {
            // If no real wasm extensions was used, we skip the initialization as it would compile
            // the placeholder extension for nothing and we have a lot of extension tests, most of
            // them not relying on wasm at all.
            Default::default()
        };

        Ok(ExtensionsDispatcher {
            wasm: wasm_extensions,
            test: TestExtensions {
                builders: self.builders,
                global_instances: self.global_instances,
                subgraph_instances: self.subgraph_instances,
            },
            dispatch: self.dispatch,
            access_log_receiver,
        })
    }
}
