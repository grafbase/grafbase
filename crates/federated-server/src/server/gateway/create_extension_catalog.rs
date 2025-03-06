use extension_catalog::{EXTENSION_DIR_NAME, Extension, ExtensionCatalog, VersionedManifest};
use gateway_config::Config;
use std::{
    env,
    fs::File,
    io,
    path::{Path, PathBuf},
};
use tokio::fs;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not load extension at {path}: {err}")]
    LoadExtension { path: String, err: String },
    #[error("{context}: {err}")]
    Io { context: String, err: io::Error },
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
    let extension_configs = gateway_config.extensions.clone().unwrap_or_default();

    if extension_configs.is_empty() {
        return Ok(catalog);
    }

    let grafbase_extensions_dir_path = cwd.join(EXTENSION_DIR_NAME);

    for (extension_name, config) in extension_configs.iter() {
        let extension = match config.path() {
            Some(path) => load_extension_from_path(&cwd.join(path), extension_name)?,
            None => {
                let extension_name_path = grafbase_extensions_dir_path.join(extension_name);
                let mut entries = fs::read_dir(&extension_name_path)
                    .await
                    .map_err(|err| Error::LoadExtension {
                        path: extension_name_path.display().to_string(),
                        err: format!("Could not read directory: {err}"),
                    })?;

                let mut relevant_entry: Option<(semver::Version, PathBuf)> = None;

                while let Some(entry) = entries.next_entry().await.map_err(|err| Error::Io {
                    context: format!("Reading entries at {}", extension_name_path.display()),
                    err,
                })? {
                    let file_type = entry.file_type().await.map_err(|err| Error::Io {
                        context: format!("Reading entries at {}", extension_name_path.display()),
                        err,
                    })?;

                    if !file_type.is_dir() {
                        continue;
                    }

                    let Ok(version) = entry
                        .file_name()
                        .to_str()
                        .unwrap_or_default()
                        .parse::<semver::Version>()
                    else {
                        continue;
                    };

                    if config.version().matches(&version)
                        && relevant_entry
                            .as_ref()
                            .map(|(existing_version, _)| existing_version < &version)
                            .unwrap_or(true)
                    {
                        relevant_entry = Some((version, entry.path()));
                    }
                }

                let Some((_, relevant_entry)) = relevant_entry else {
                    return Err(Error::LoadExtension {
                        path: extension_name_path.display().to_string(),
                        err: "No matching version of the extension found.".to_string(),
                    });
                };

                load_extension_from_path(&relevant_entry, extension_name)?
            }
        };

        catalog.push(extension);
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
    use extension_catalog::{ExtensionCatalog, Manifest};

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
            kind: extension_catalog::Kind::Resolver(extension_catalog::ResolverKind {
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
           path = "./test1"

           [extensions.test_two]
           version = "0.20.0"
           path = "./test_two"
        "#;

        let dir = tempfile::tempdir().expect("Failed to create temporary directory");

        let err = run_test(dir.path(), config).unwrap_err();

        if cfg!(windows) {
            return; // different error message
        }

        let err = err
            .to_string()
            .replace(&dir.path().display().to_string(), "<tmp-dir-path>");

        insta::assert_snapshot!(err, @"could not load extension at <tmp-dir-path>/./test1: Could not read directory: No such file or directory (os error 2)");
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
    fn with_versions_existing() {
        let config = r#"
           [extensions.test_one]
           version = "0.1.0"

           [extensions.test_two]
           version = "0.20.0"
        "#;

        let dir = tempfile::tempdir().expect("Failed to create temporary directory");

        // Create test1 directory and necessary files
        let test1_dir = dir.path().join("grafbase_extensions/test_one/0.1.2");
        std::fs::create_dir_all(&test1_dir).expect("Failed to create test1 directory");

        // Create manifest.json
        let manifest = make_manifest("test_one", "0.1.0");
        let manifest_json = serde_json::to_string_pretty(&manifest).expect("Failed to serialize manifest");
        std::fs::write(test1_dir.join("manifest.json"), manifest_json).expect("Failed to write manifest.json");

        // Create empty extension.wasm file
        std::fs::write(test1_dir.join("extension.wasm"), []).expect("Failed to write extension.wasm");

        let test2_dir = dir.path().join("grafbase_extensions/test_two/0.20.0");
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
    fn with_versions_non_compatible() {
        let config = r#"
           [extensions.test_one]
           version = "0.1.0"

           [extensions.test_two]
           version = "0.20.0"
        "#;

        let dir = tempfile::tempdir().expect("Failed to create temporary directory");

        // Create test1 directory and necessary files
        let test1_dir = dir.path().join("grafbase_extensions/test_one/0.1.2");
        std::fs::create_dir_all(&test1_dir).expect("Failed to create test1 directory");

        // Create manifest.json
        let manifest = make_manifest("test_one", "0.1.0");
        let manifest_json = serde_json::to_string_pretty(&manifest).expect("Failed to serialize manifest");
        std::fs::write(test1_dir.join("manifest.json"), manifest_json).expect("Failed to write manifest.json");

        // Create empty extension.wasm file
        std::fs::write(test1_dir.join("extension.wasm"), []).expect("Failed to write extension.wasm");

        let test2_dir = dir.path().join("grafbase_extensions/test_two/0.19.0");
        std::fs::create_dir_all(&test2_dir).expect("Failed to create test1 directory");

        // Create manifest.json
        let manifest = make_manifest("test_two", "0.1.0");
        let manifest_json = serde_json::to_string_pretty(&manifest).expect("Failed to serialize manifest");
        std::fs::write(test2_dir.join("manifest.json"), manifest_json).expect("Failed to write manifest.json");

        // Create empty extension.wasm file
        std::fs::write(test2_dir.join("extension.wasm"), []).expect("Failed to write extension.wasm");

        let err = run_test(dir.path(), config).unwrap_err();

        if cfg!(windows) {
            return; // different error message
        }

        let err = err
            .to_string()
            .replace(&dir.path().display().to_string(), "<tmp-dir-path>");

        insta::assert_debug_snapshot!(err, @r#""could not load extension at <tmp-dir-path>/grafbase_extensions/test_two: No matching version of the extension found.""#);
    }
}
