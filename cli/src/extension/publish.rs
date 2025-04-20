use crate::{backend::api, cli_input::ExtensionPublishCommand, output::report};
use extension::VersionedManifest;
use std::fs;

use super::EXTENSION_WASM_MODULE_FILE_NAME;

pub(super) async fn execute(cmd: ExtensionPublishCommand) -> anyhow::Result<()> {
    let manifest_path = cmd.path.join("manifest.json");
    let manifest_reader = fs::File::open(&manifest_path).map_err(|err| {
        anyhow::anyhow!(
            "Failed to open extension manifest at `{}`: {err}",
            manifest_path
                // Get the full path in the error when things go wrong.
                .canonicalize()
                .ok()
                .as_deref()
                .unwrap_or(&manifest_path)
                .display()
        )
    })?;

    let manifest: VersionedManifest = serde_json::from_reader(manifest_reader).map_err(|err| {
        anyhow::anyhow!(
            "Failed to parse extension manifest at `{}`: {err}",
            manifest_path.display()
        )
    })?;

    let wasm_blob_path = cmd.path.join(EXTENSION_WASM_MODULE_FILE_NAME);

    if !wasm_blob_path.exists() {
        return Err(anyhow::anyhow!(
            "Failed to find extension WASM module at `{}`",
            wasm_blob_path.display()
        ));
    }

    match api::extension_publish::extension_publish(manifest, &wasm_blob_path).await? {
        api::extension_publish::ExtensionPublishOutcome::Success { name, version } => {
            report::extension_published(&name, &version);
        }
        api::extension_publish::ExtensionPublishOutcome::BadWasmModuleError(err)
        | api::extension_publish::ExtensionPublishOutcome::ExtensionValidationError(err) => {
            report::extension_publish_failed(&err);
            std::process::exit(1);
        }
        api::extension_publish::ExtensionPublishOutcome::VersionAlreadyExists => {
            report::extension_version_already_exists();
            std::process::exit(1);
        }
    }

    Ok(())
}
