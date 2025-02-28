use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Stdio,
};

use anyhow::Context;
use extension::{FieldResolver, Kind, Manifest};
use semver::Version;
use serde_valid::Validate;

use crate::{cli_input::ExtensionBuildCommand, output::report};

use super::EXTENSION_WASM_MODULE_FILE_NAME;

const RUST_TARGET: &str = "wasm32-wasip2";

pub(crate) fn execute(cmd: ExtensionBuildCommand) -> anyhow::Result<()> {
    let output_dir = cmd.output_dir;
    let scratch_dir = cmd.scratch_dir;
    let source_dir = cmd.source_dir;
    let debug_mode = cmd.debug;

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

#[derive(serde::Deserialize)]
struct ExtensionToml {
    extension: ExtensionTomlExtension,
    #[serde(default)]
    directives: ExtensionTomlDirectives,
}

#[derive(serde::Deserialize, Validate)]
struct ExtensionTomlExtension {
    #[validate(pattern = "^[a-z0-9-]+$")]
    name: String,
    version: Version,
    kind: ExtensionKind,
    description: String,
    homepage_url: Option<url::Url>,
    repository_url: Option<url::Url>,
    license: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum ExtensionKind {
    Resolver,
    Auth,
}

#[derive(Default, serde::Deserialize)]
struct ExtensionTomlDirectives {
    definitions: Option<String>,
    field_resolvers: Option<Vec<String>>,
}

struct Versions {
    minimum_gateway_version: Version,
    sdk_version: Version,
}

fn check_rust() -> anyhow::Result<()> {
    let rustup = new_command("rustup").arg("--version").output()?;

    if !rustup.status.success() {
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

    let output = new_command("cargo")
        .args(["build", "--target", RUST_TARGET])
        .args(if debug_mode { None } else { Some("--release") })
        // disable sscache, if enabled. does not work with wasi builds :P
        .env("RUSTC_WRAPPER", "")
        .env("CARGO_TARGET_DIR", scratch_dir)
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

    let mut wasm_path = scratch_dir.to_path_buf();

    wasm_path.extend([
        RUST_TARGET,
        if debug_mode { "debug" } else { "release" },
        &cargo_toml.package.name.replace('-', "_"),
    ]);

    wasm_path.set_extension("wasm");

    Ok(wasm_path)
}

fn parse_manifest(source_dir: &Path, wasm_path: &Path) -> anyhow::Result<Manifest> {
    let extension_toml = std::fs::read_to_string(source_dir.join("extension.toml"))
        .context("could not find extension.toml file from the extension project")?;

    let extension_toml: ExtensionToml =
        toml::from_str(&extension_toml).map_err(|e| anyhow::anyhow!("extension.toml contains invalid data\n{e}"))?;

    let wasm_bytes =
        std::fs::read(wasm_path).with_context(|| format!("failed to read extension `{}`", wasm_path.display()))?;

    let versions = parse_versions(&wasm_bytes)?;

    let kind = match extension_toml.extension.kind {
        ExtensionKind::Resolver => {
            let resolver_directives = extension_toml.directives.field_resolvers.unwrap_or_default();

            Kind::FieldResolver(FieldResolver { resolver_directives })
        }
        ExtensionKind::Auth => Kind::Authenticator(Default::default()),
    };

    let sdl = match extension_toml.directives.definitions.map(|path| source_dir.join(&path)) {
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

    let manifest = Manifest {
        id: extension::Id {
            name: extension_toml.extension.name,
            version: extension_toml.extension.version,
        },
        kind,
        sdk_version: versions.sdk_version,
        minimum_gateway_version: versions.minimum_gateway_version,
        sdl,
        description: extension_toml.extension.description,
        readme: None,
        homepage_url: extension_toml.extension.homepage_url,
        repository_url: extension_toml.extension.repository_url,
        license: extension_toml.extension.license,
    };

    Ok(manifest)
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
fn new_command(program: impl AsRef<OsStr>) -> std::process::Command {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000_u32;

    let mut command = std::process::Command::new(program);
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

#[cfg(not(target_os = "windows"))]
fn new_command(program: impl AsRef<OsStr>) -> std::process::Command {
    std::process::Command::new(program)
}
