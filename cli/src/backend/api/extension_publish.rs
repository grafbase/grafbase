use crate::common::consts::GRAFBASE_API_URL_ENV_VAR;

use super::{client::create_client, graphql::mutations::extension_publish};
use cynic::MutationBuilder;
use extension::VersionedManifest;
use std::path::Path;

pub(crate) enum ExtensionPublishOutcome {
    Success { name: String, version: String },
    VersionAlreadyExists,
    ExtensionValidationError(String),
    BadWasmModuleError(String),
}

// FIXME: this is a temporary state of affairs. The gateway does not support GraphQL multipart uploads yet. Issue: GB-8590.
const API_URL_FOR_EXTENSION_REGISTRY: &str = "https://api.ep.grafbase.com/graphql";

pub(crate) async fn extension_publish(
    manifest: VersionedManifest,
    wasm_blob_path: &Path,
) -> anyhow::Result<ExtensionPublishOutcome> {
    let client = create_client()?;

    let operation = extension_publish::ExtensionPublish::build(extension_publish::ExtensionPublishVariables {
        manifest,
        wasm_module: extension_publish::Upload,
    });

    let form = reqwest::multipart::Form::new()
        .text("operations", serde_json::to_string(&operation).unwrap())
        .text("map", serde_json::json!({ "0": ["variables.wasmModule"]}).to_string())
        .file("0", wasm_blob_path)
        .await?;

    let api_url = std::env::var(GRAFBASE_API_URL_ENV_VAR).unwrap_or_else(|_| API_URL_FOR_EXTENSION_REGISTRY.to_owned());

    let req = client.post(&api_url).multipart(form);

    let response = req.send().await?;
    let status = response.status();

    if !status.is_success() {
        let text = response.text().await?;
        return Err(anyhow::anyhow!(
            "Failed to publish extension. Status: {status}. Error: {text}"
        ));
    }

    let response: cynic::GraphQlResponse<extension_publish::ExtensionPublish> = response.json().await?;

    let Some(payload) = response.data.and_then(|data| data.extension_publish) else {
        return Err(anyhow::anyhow!("GraphQL response error: {:?}", response.errors));
    };

    match payload {
        extension_publish::ExtensionPublishPayload::ExtensionPublishSuccess(
            extension_publish::ExtensionPublishSuccess {
                extension_version:
                    extension_publish::ExtensionVersion {
                        extension: extension_publish::Extension { name },
                        version,
                    },
            },
        ) => Ok(ExtensionPublishOutcome::Success {
            name,
            version: version.to_string(),
        }),
        extension_publish::ExtensionPublishPayload::ExtensionVersionAlreadyExistsError(_) => {
            Ok(ExtensionPublishOutcome::VersionAlreadyExists)
        }
        extension_publish::ExtensionPublishPayload::BadWasmModuleError(bad_wasm_module_error) => {
            Ok(ExtensionPublishOutcome::BadWasmModuleError(bad_wasm_module_error.error))
        }
        extension_publish::ExtensionPublishPayload::ExtensionValidationError(extension_validation_error) => Ok(
            ExtensionPublishOutcome::ExtensionValidationError(extension_validation_error.error),
        ),
        extension_publish::ExtensionPublishPayload::Unknown(variant) => {
            Err(anyhow::anyhow!("Unexpected response: {variant}"))
        }
    }
}
