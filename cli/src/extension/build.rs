mod extension_toml;

use std::{
    fs,
    path::{Path, PathBuf},
    process::Stdio,
};

use anyhow::Context;
use extension::{ExtensionPermission, FieldResolverType, Manifest, Type};
use extension_toml::{ExtensionToml, ExtensionType};
use semver::Version;

use crate::{cli_input::ExtensionBuildCommand, output::report, watercolor};

use super::EXTENSION_WASM_MODULE_FILE_NAME;

const RUST_TARGET: &str = "wasm32-wasip2";

pub(crate) fn execute(cmd: ExtensionBuildCommand) -> anyhow::Result<()> {
    let output_dir = cmd.output_dir;
    let mut scratch_dir = cmd.scratch_dir;
    let source_dir = cmd.source_dir;
    let debug_mode = cmd.debug;

    // If scratch_dir is the default "./target" and source_dir is not ".",
    // make scratch_dir relative to source_dir but as an absolute path
    if scratch_dir.as_os_str() == "./target" && source_dir.as_os_str() != "." {
        let current_dir = std::env::current_dir().context("failed to get current directory")?;
        scratch_dir = current_dir.join(&source_dir).join("target");
    }

    if !output_dir.exists() {
        std::fs::create_dir_all(&output_dir).context("failed to create the output directory")?;
    }

    if !output_dir.is_dir() {
        anyhow::bail!("output path '{}' is not a directory", output_dir.display());
    }

    check_rust()?;
    install_wasm_target_if_needed()?;

    let wasm_path = compile_extension(debug_mode, &scratch_dir, &source_dir, &output_dir)?;
    let manifest = parse_manifest(&source_dir, &wasm_path)?;

    std::fs::rename(wasm_path, output_dir.join(EXTENSION_WASM_MODULE_FILE_NAME)).context("failed to move wasm file")?;
    std::fs::write(
        output_dir.join("manifest.json"),
        serde_json::to_vec(&manifest.clone().into_versioned()).unwrap(),
    )
    .context("failed to write manifest file")?;

    report::extension_built(&manifest);

    Ok(())
}

#[derive(serde::Deserialize)]
struct CargoToml {
    package: CargoTomlPackage,
}

#[derive(serde::Deserialize)]
struct CargoTomlPackage {
    name: String,
}

struct Versions {
    minimum_gateway_version: Version,
    sdk_version: Version,
}

fn check_rust() -> anyhow::Result<()> {
    let rustup_exists = new_command("rustup")
        .arg("--version")
        .output()
        .map(|result| result.status.success())
        .unwrap_or_default();

    if !rustup_exists {
        anyhow::bail!(
            "A working rustup installation is required to build extensions. Please install it from https://rustup.rs/ before continuing."
        );
    }

    let rustc_version = new_command("rustc").arg("--version").output()?;

    if !rustc_version.status.success() {
        anyhow::bail!(
            "Failed to run rustc: {}",
            String::from_utf8_lossy(&rustc_version.stderr)
        );
    }

    let output = String::from_utf8_lossy(&rustc_version.stdout).to_string();

    let Some(output) = output.split(' ').nth(1) else {
        anyhow::bail!("failed to parse rustc version");
    };

    let version = Version::parse(output).context("failed to parse rustc version")?;

    if version < Version::new(1, 82, 0) {
        anyhow::bail!(
            "Rust version 1.82.0 or newer is required to build extensions. Please update your Rust installation before continuing."
        );
    }

    Ok(())
}

fn install_wasm_target_if_needed() -> anyhow::Result<()> {
    let rustc_exists = new_command("rustc")
        .arg("--version")
        .output()
        .map(|result| result.status.success())
        .unwrap_or_default();

    if !rustc_exists {
        anyhow::bail!(
            "Rust must be installed to build extensions. Please install it from https://rustup.rs/ before continuing."
        );
    }

    let output = new_command("rustc")
        .arg("--print")
        .arg("sysroot")
        .output()
        .context("failed to run rustc")?;

    if !output.status.success() {
        anyhow::bail!(
            "failed to retrieve rust sysroot: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let sysroot = PathBuf::from(String::from_utf8(output.stdout)?.trim());

    if sysroot.join("lib/rustlib").join(RUST_TARGET).exists() {
        return Ok(());
    }

    let output = new_command("rustup")
        .args(["target", "add", RUST_TARGET])
        .stderr(Stdio::piped())
        .stdout(Stdio::inherit())
        .output()
        .context("failed to run `rustup target add`")?;

    if !output.status.success() {
        anyhow::bail!(
            "failed to install `{RUST_TARGET}` target: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

fn compile_extension(
    debug_mode: bool,
    scratch_dir: &Path,
    source_dir: &Path,
    output_dir: &Path,
) -> anyhow::Result<PathBuf> {
    report::extension_build_start();

    // Ensure scratch_dir is absolute to prevent nested directories when cargo changes to source_dir
    let absolute_scratch_dir = if scratch_dir.is_absolute() {
        scratch_dir.to_path_buf()
    } else {
        std::env::current_dir()
            .context("failed to get current directory")?
            .join(scratch_dir)
    };

    let output = new_command("cargo")
        .args(["build", "--target", RUST_TARGET])
        .args(if debug_mode { None } else { Some("--release") })
        // disable sscache, if enabled. does not work with wasi builds :P
        .env("RUSTC_WRAPPER", "")
        .env("CARGO_TARGET_DIR", &absolute_scratch_dir)
        .current_dir(source_dir)
        .stderr(Stdio::piped())
        .stdout(Stdio::inherit())
        .output()
        .context("failed to run `cargo`")?;

    if !output.status.success() {
        anyhow::bail!(
            "failed to compile extension: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    if !std::fs::exists(output_dir)? {
        std::fs::create_dir_all(output_dir).context("failed to create the output directory")?;
    }

    let cargo_toml = std::fs::read_to_string(source_dir.join("Cargo.toml"))?;
    let cargo_toml: CargoToml = toml::from_str(&cargo_toml)?;

    let mut wasm_path = absolute_scratch_dir.clone();

    wasm_path.extend([
        RUST_TARGET,
        if debug_mode { "debug" } else { "release" },
        &cargo_toml.package.name.replace('-', "_"),
    ]);

    wasm_path.set_extension("wasm");

    Ok(wasm_path)
}

fn parse_manifest(source_dir: &Path, wasm_path: &Path) -> anyhow::Result<Manifest> {
    let extension_toml_path = std::fs::read_to_string(source_dir.join("extension.toml"))
        .context("could not find extension.toml file from the extension project")?;

    let toml: ExtensionToml = toml::from_str(&extension_toml_path)
        .map_err(|e| anyhow::anyhow!("extension.toml contains invalid data\n{e}"))?;

    let wasm_bytes =
        std::fs::read(wasm_path).with_context(|| format!("failed to read extension `{}`", wasm_path.display()))?;

    let versions = parse_versions(&wasm_bytes)?;

    let extension_type = match toml.extension.r#type {
        // == Legacy types ==
        ExtensionType::Resolver if versions.sdk_version < Version::new(0, 17, 0) => {
            Type::FieldResolver(FieldResolverType {
                resolver_directives: toml.legacy_directives.field_resolvers,
            })
        }
        ExtensionType::SelectionSetResolver => Type::SelectionSetResolver(Default::default()),
        // == Current types ==
        ExtensionType::Resolver => {
            #[derive(serde::Serialize)]
            struct NewFormat {
                resolver: extension_toml::ResolverType,
            }

            Type::Resolver(if let Some(res) = toml.resolver {
                extension::ResolverType {
                    directives: res.directives,
                }
            } else if let Some(directives) = toml.legacy_directives.resolvers.clone() {
                let new_toml = toml::to_string_pretty(&NewFormat {
                    resolver: extension_toml::ResolverType {
                        directives: Some(directives.clone()),
                    },
                })
                .unwrap();
                watercolor::output!("⚠️ Warning: 'directives.resolvers' is deprecated, instead use:\n{new_toml}", @BrightYellow);
                extension::ResolverType {
                    directives: Some(directives),
                }
            } else {
                Default::default()
            })
        }
        ExtensionType::Authentication => Type::Authentication(Default::default()),
        ExtensionType::Authorization => {
            #[derive(serde::Serialize)]
            struct NewFormat {
                authorization: extension_toml::AuthorizationType,
            }

            Type::Authorization(if let Some(res) = toml.authorization {
                extension::AuthorizationType {
                    directives: res.directives,
                    group_by: res.group_by,
                }
            } else if let Some(directives) = toml.legacy_directives.authorization.clone() {
                let new_toml = toml::to_string_pretty(&NewFormat {
                    authorization: extension_toml::AuthorizationType {
                        directives: Some(directives.clone()),
                        group_by: None,
                    },
                })
                .unwrap();
                watercolor::output!("⚠️ Warning: 'directives.authorization' is deprecated, instead use:\n{new_toml}", @BrightYellow);
                extension::AuthorizationType {
                    directives: Some(directives),
                    group_by: None,
                }
            } else {
                Default::default()
            })
        }
        ExtensionType::Hooks => Type::Hooks(if let Some(hooks) = toml.hooks {
            extension::HooksType {
                event_filter: hooks.events.map(Into::into),
            }
        } else {
            Default::default()
        }),
        ExtensionType::Contracts => Type::Contracts(Default::default()),
    };

    let sdl_path = toml
        .legacy_directives
        .definitions
        .map(|path| {
                watercolor::output!("⚠️ Warning: Specifying 'directives.definitions' is deprecated. GraphQL directives will always be expected to be in 'definitions.graphql' in the future.", @BrightYellow);
    source_dir.join(&path)
        })
        .or_else(|| {
            let path = source_dir.join("definitions.graphql");
            path.exists().then_some(path)
        });

    let sdl = match sdl_path {
        Some(ref path) => {
            let Ok(sdl) = std::fs::read_to_string(path) else {
                anyhow::bail!("failed to read directive definitions in {}", path.display())
            };

            if let Err(e) = cynic_parser::parse_type_system_document(&sdl) {
                println!("{}", e.to_report(&sdl));
                anyhow::bail!("failed to parse directive definitions in {}", path.display());
            };

            Some(sdl)
        }
        None => None,
    };

    let mut permissions = Vec::new();

    if toml.permissions.network {
        permissions.push(ExtensionPermission::Network);
    }

    if toml.permissions.stdout {
        permissions.push(ExtensionPermission::Stdout);
    }

    if toml.permissions.stderr {
        permissions.push(ExtensionPermission::Stderr);
    }

    if toml.permissions.environment_variables {
        permissions.push(ExtensionPermission::EnvironmentVariables);
    }

    let manifest = Manifest {
        id: extension::Id {
            name: toml.extension.name,
            version: toml.extension.version,
        },
        r#type: extension_type,
        sdk_version: versions.sdk_version,
        minimum_gateway_version: versions.minimum_gateway_version,
        sdl,
        description: toml.extension.description,
        readme: try_get_readme(source_dir),
        homepage_url: toml.extension.homepage_url,
        repository_url: toml.extension.repository_url,
        license: toml.extension.license,
        permissions,
        legacy_event_filter: None,
    };

    Ok(manifest)
}

fn try_get_readme(source_dir: &Path) -> Option<String> {
    let entries = fs::read_dir(source_dir).ok()?;

    for entry in entries.filter_map(|entry| entry.ok()) {
        let Some(file_type) = entry.file_type().ok() else {
            continue;
        };

        if !file_type.is_file() {
            continue;
        };

        let file_name = entry.file_name();

        let Some(file_name) = file_name.to_str() else {
            continue;
        };

        if file_name.eq_ignore_ascii_case("readme.md") {
            if let Ok(readme) = fs::read_to_string(entry.path()) {
                return Some(readme);
            }
        }
    }

    None
}

fn parse_versions(wasm_bytes: &[u8]) -> anyhow::Result<Versions> {
    let mut minimum_gateway_version = None;
    let mut sdk_version = None;

    for part in wasmparser::Parser::new(0).parse_all(wasm_bytes) {
        if let wasmparser::Payload::CustomSection(custom) = part.context("error parsing extension")? {
            if custom.name() == "sdk:minimum-gateway-version" {
                minimum_gateway_version = parse_version_custom_section(custom.data());

                if minimum_gateway_version.is_none() {
                    anyhow::bail!("extension has invalid sdk:minimum-gateway-version section");
                }
            } else if custom.name() == "sdk:version" {
                sdk_version = parse_version_custom_section(custom.data());

                if sdk_version.is_none() {
                    anyhow::bail!("extension has invalid sdk:version section");
                }
            }
        }
    }

    let minimum_gateway_version = minimum_gateway_version
        .ok_or_else(|| anyhow::anyhow!("extension has no sdk:minimum-gateway-version section"))?;

    let sdk_version = sdk_version.ok_or_else(|| anyhow::anyhow!("extension has no missing sdk:version section"))?;

    Ok(Versions {
        minimum_gateway_version,
        sdk_version,
    })
}

fn parse_version_custom_section(data: &[u8]) -> Option<Version> {
    if data.len() != 6 {
        return None;
    }

    Some(Version::new(
        u16::from_be_bytes([data[0], data[1]]) as u64,
        u16::from_be_bytes([data[2], data[3]]) as u64,
        u16::from_be_bytes([data[4], data[5]]) as u64,
    ))
}

#[cfg(target_os = "windows")]
fn new_command(program: &'static str) -> std::process::Command {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000_u32;

    let mut command = std::process::Command::new(program);
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

#[cfg(not(target_os = "windows"))]
fn new_command(program: &'static str) -> std::process::Command {
    std::process::Command::new(program)
}
