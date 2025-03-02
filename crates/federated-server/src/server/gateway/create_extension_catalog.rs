use extension_catalog::{Extension, ExtensionCatalog, PUBLIC_EXTENSION_REGISTRY_URL, VersionedManifest};
use gateway_config::Config;
use std::{env, fs::File, io, path::Path};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not load extension at {path}: {err}")]
    LoadExtension { path: String, err: String },
    #[error("could not read extension lockfile: {0}")]
    ReadExtensionLockfile(String),
    #[error("{context}: {err}")]
    Io { context: String, err: io::Error },
    #[error("extension `{0}` is missing from lockfile")]
    MissingFromLockfile(String),
    #[error("failed to download extension: {0}")]
    Download(extension_catalog::Report),
}

pub(super) async fn create_extension_catalog(gateway_config: &Config) -> Result<ExtensionCatalog, Error> {
    let cwd = env::current_dir().map_err(|e| Error::Io {
        context: "Failed to get current directory".to_string(),
        err: e,
    })?;

    create_extension_catalog_impl(gateway_config, &cwd).await
}

async fn create_extension_catalog_impl(gateway_config: &Config, cwd: &Path) -> Result<ExtensionCatalog, Error> {
    let mut catalog = ExtensionCatalog::default();

    let Some(extension_configs) = &gateway_config.extensions else {
        return Ok(catalog);
    };

    let grafbase_extensions_dir_path = cwd.join("grafbase_extensions");
    let lockfile_path = cwd.join("grafbase-extensions.lock");
    let mut lockfile: Option<extension_catalog::lockfile::Lockfile> = None;
    let mut extensions_dir_exists = grafbase_extensions_dir_path.exists();
    let http_client = reqwest::Client::new();

    for (extension_name, config) in extension_configs.iter() {
        if let Some(path) = config.path() {
            match load_extension_from_path(&cwd.join(path), extension_name) {
                Ok(extension) => {
                    catalog.push(extension);
                    continue;
                }
                Err(err) => {
                    tracing::warn!("Failed to load extension from path. {err}");
                }
            }
        }

        if lockfile.is_none() {
            let lockfile_str = tokio::fs::read_to_string(&lockfile_path)
                .await
                .map_err(|err| Error::ReadExtensionLockfile(err.to_string()))?;

            lockfile =
                Some(toml::from_str(&lockfile_str).map_err(|err| Error::ReadExtensionLockfile(err.to_string()))?);
        }

        let lockfile = lockfile.as_ref().expect("Lockfile to be initialized");

        if !extensions_dir_exists {
            std::fs::create_dir_all(&grafbase_extensions_dir_path).map_err(|err| Error::Io {
                context: "Creating grafbase_extensions directory".to_owned(),
                err,
            })?;
            extensions_dir_exists = true;
        }

        let Some(extension_in_lockfile) = lockfile.extensions.iter().find(|extension| {
            extension.name.as_str() == extension_name && config.version().matches(&extension.version)
        }) else {
            return Err(Error::MissingFromLockfile(extension_name.clone()));
        };

        let extension_path = grafbase_extensions_dir_path
            .join(extension_name)
            .join(extension_in_lockfile.version.to_string());

        if let Ok(extension) = load_extension_from_path(&extension_path, extension_name) {
            catalog.push(extension);
            continue;
        };

        let base_registry_url = std::env::var("EXTENSION_REGISTRY_URL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| PUBLIC_EXTENSION_REGISTRY_URL.parse().unwrap());

        extension_catalog::download_extension_from_registry(
            &http_client,
            &grafbase_extensions_dir_path,
            extension_name.clone(),
            extension_in_lockfile.version.clone(),
            &base_registry_url,
        )
        .await
        .map_err(Error::Download)?;

        load_extension_from_path(&extension_path, extension_name)?;
    }

    Ok(catalog)
}

fn load_extension_from_path(path: &std::path::Path, expected_extension_name: &str) -> Result<Extension, Error> {
    let extension_dir = path.read_dir().map_err(|err| Error::LoadExtension {
        path: path.display().to_string(),
        err: format!("Could not read directory: {err}"),
    })?;

    let mut manifest = None;
    let mut wasm_path = None;

    for entry in extension_dir {
        let entry = entry.map_err(|err| Error::Io {
            context: "Reading grafbase_extensions directory".to_owned(),
            err,
        })?;

        match entry.file_name().as_encoded_bytes() {
            b"extension.wasm" => {
                wasm_path = Some(path.join("extension.wasm"));
            }
            b"manifest.json" => {
                let manifest_data = File::open(path.join("manifest.json")).map_err(|err| Error::Io {
                    context: "Loading manifest.json".to_owned(),
                    err,
                })?;

                let versioned_manifest: VersionedManifest =
                    serde_json::from_reader(manifest_data).map_err(|err| Error::LoadExtension {
                        path: path.display().to_string(),
                        err: format!("Could not parse manifest json: {err}"),
                    })?;
                manifest = Some(versioned_manifest);
            }
            other => {
                return Err(Error::LoadExtension {
                    path: path.display().to_string(),
                    err: format!("Found unknown file `{}`", String::from_utf8_lossy(other)),
                });
            }
        }
    }

    let Some((manifest, wasm_path)) = manifest.zip(wasm_path) else {
        return Err(Error::LoadExtension {
            path: path.display().to_string(),
            err: "Missing manifest.json or extension.wasm".to_string(),
        });
    };

    let manifest = manifest.into_latest();

    if manifest.id.name != expected_extension_name {
        return Err(Error::LoadExtension {
            path: path.display().to_string(),
            err: format!(
                "Expected extension `{expected_extension_name}` but found `{found}`",
                found = manifest.id.name
            ),
        });
    }

    Ok(Extension {
        manifest,
        wasm_path: wasm_path
            .canonicalize()
            .expect("Failed to canonicalize extension.wasm path"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use extension_catalog::{ExtensionCatalog, Manifest, lockfile};

    fn run_test(cwd: &Path, config: &str) -> Result<ExtensionCatalog, Error> {
        let rt = tokio::runtime::Builder::new_multi_thread().build().unwrap();
        let config = toml::from_str(config).unwrap();

        rt.block_on(create_extension_catalog_impl(&config, cwd))
    }

    fn make_manifest(name: &str, version: &str) -> VersionedManifest {
        VersionedManifest::V1(Manifest {
            id: extension_catalog::Id {
                name: name.to_string(),
                version: version.parse().unwrap(),
            },
            kind: extension_catalog::Kind::FieldResolver(extension_catalog::FieldResolver {
                resolver_directives: None,
            }),
            sdk_version: "0.1.0".parse().unwrap(),
            minimum_gateway_version: "0.1.0".parse().unwrap(),
            description: "my extension".to_owned(),
            sdl: None,
            readme: None,
            homepage_url: None,
            repository_url: None,
            license: None,
            permissions: Default::default(),
        })
    }

    #[test]
    fn no_extensions() {
        let config = r#"
           [graph]
           introspection = true
        "#;

        let dir = tempfile::tempdir().expect("Failed to create temporary directory");

        let catalog = run_test(dir.path(), config).unwrap();

        assert!(catalog.iter().next().is_none());
    }

    #[test]
    fn with_paths_missing() {
        let config = r#"
           [extensions.test_one]
           version = "0.1.0"

           [extensions.test_two]
           version = "0.20.0"
        "#;

        let dir = tempfile::tempdir().expect("Failed to create temporary directory");

        let err = run_test(dir.path(), config).unwrap_err();

        if cfg!(windows) {
            return; // different error message
        }

        insta::assert_snapshot!(err, @"could not read extension lockfile: No such file or directory (os error 2)");
    }

    #[test]
    fn with_paths_existing() {
        let config = r#"
           [extensions.test_one]
           version = "0.1.0"
           path = "./test1"

           [extensions.test_two]
           version = "0.20.0"
           path = "./test_two"
        "#;

        let dir = tempfile::tempdir().expect("Failed to create temporary directory");

        assert!(dir.path().exists());

        // Create test1 directory and necessary files
        let test1_dir = dir.path().join("test1");
        std::fs::create_dir_all(&test1_dir).expect("Failed to create test1 directory");

        // Create manifest.json
        let manifest = make_manifest("test_one", "0.1.0");
        let manifest_json = serde_json::to_string_pretty(&manifest).expect("Failed to serialize manifest");
        std::fs::write(test1_dir.join("manifest.json"), manifest_json).expect("Failed to write manifest.json");

        // Create empty extension.wasm file
        std::fs::write(test1_dir.join("extension.wasm"), []).expect("Failed to write extension.wasm");

        let test2_dir = dir.path().join("test_two");
        std::fs::create_dir_all(&test2_dir).expect("Failed to create test1 directory");

        // Create manifest.json
        let manifest = make_manifest("test_two", "0.1.0");
        let manifest_json = serde_json::to_string_pretty(&manifest).expect("Failed to serialize manifest");
        std::fs::write(test2_dir.join("manifest.json"), manifest_json).expect("Failed to write manifest.json");

        // Create empty extension.wasm file
        std::fs::write(test2_dir.join("extension.wasm"), []).expect("Failed to write extension.wasm");

        let catalog = run_test(dir.path(), config).unwrap();

        let extensions = catalog.iter().map(|ext| &ext.manifest.id).collect::<Vec<_>>();

        insta::assert_debug_snapshot!(extensions, @r#"
        [
            Id {
                name: "test_one",
                version: Version {
                    major: 0,
                    minor: 1,
                    patch: 0,
                },
            },
            Id {
                name: "test_two",
                version: Version {
                    major: 0,
                    minor: 1,
                    patch: 0,
                },
            },
        ]
        "#);
    }

    #[test]
    fn with_versions_no_lockfile() {
        let config = r#"
           [extensions.test_one]
           version = "0.1.0"

           [extensions.test_two]
           version = "0.20.0"
        "#;

        let dir = tempfile::tempdir().expect("Failed to create temporary directory");

        let err = run_test(dir.path(), config).unwrap_err();

        if cfg!(windows) {
            return; // different error message
        }

        insta::assert_snapshot!(err, @"could not read extension lockfile: No such file or directory (os error 2)");
    }

    #[test]
    fn with_lockfile_and_downloaded_extension() {
        let dir = tempfile::tempdir().expect("Failed to create temporary directory");

        let lockfile = lockfile::VersionedLockfile::V1(lockfile::Lockfile {
            extensions: vec![lockfile::Extension {
                name: "test_one".to_owned(),
                version: "0.1.3".parse().unwrap(),
            }],
        });

        let lockfile_path = dir.path().join("grafbase-extensions.lock");

        let lockfile_string = toml::ser::to_string(&lockfile).unwrap();

        std::fs::write(lockfile_path, lockfile_string.as_bytes()).unwrap();

        let config = r#"
           [extensions.test_one]
           version = "0.1.0"
        "#;

        // Create test1 directory and necessary files
        let test1_dir = dir.path().join("grafbase_extensions/test_one/0.1.3");
        std::fs::create_dir_all(&test1_dir).expect("Failed to create test1 directory");

        // Create manifest.json
        let manifest = make_manifest("test_one", "0.1.0");
        let manifest_json = serde_json::to_string_pretty(&manifest).expect("Failed to serialize manifest");
        std::fs::write(test1_dir.join("manifest.json"), manifest_json).expect("Failed to write manifest.json");

        // Create empty extension.wasm file
        std::fs::write(test1_dir.join("extension.wasm"), []).expect("Failed to write extension.wasm");

        let catalog = run_test(dir.path(), config).unwrap();

        let extensions = catalog.iter().map(|ext| &ext.manifest.id).collect::<Vec<_>>();

        insta::assert_debug_snapshot!(extensions, @r#"
        [
            Id {
                name: "test_one",
                version: Version {
                    major: 0,
                    minor: 1,
                    patch: 0,
                },
            },
        ]
        "#);
    }
}
