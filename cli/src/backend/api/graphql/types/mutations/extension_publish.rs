use super::schema;
use cynic::impl_scalar;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub(crate) struct Upload;

impl_scalar!(Upload, schema::Upload);
impl_scalar!(semver::Version, schema::SemverVersion);
impl_scalar!(extension::VersionedManifest, schema::VersionedExtensionManifest);

#[derive(Debug, cynic::QueryFragment)]
pub struct Extension {
    pub name: String,
}

#[derive(Debug, cynic::QueryFragment)]
pub struct ExtensionVersion {
    pub extension: Extension,
    pub version: semver::Version,
}

#[derive(Debug, cynic::QueryFragment)]
pub struct ExtensionPublishSuccess {
    pub extension_version: ExtensionVersion,
}

#[derive(Debug, cynic::QueryFragment)]
pub struct BadWasmModuleError {
    pub error: String,
}

#[derive(Debug, cynic::QueryFragment)]
pub struct ExtensionValidationError {
    pub error: String,
}

#[derive(Debug, cynic::QueryFragment)]
pub struct ExtensionVersionAlreadyExistsError {
    __typename: String,
}

#[derive(Debug, cynic::InlineFragments)]
pub enum ExtensionPublishPayload {
    ExtensionPublishSuccess(ExtensionPublishSuccess),
    BadWasmModuleError(BadWasmModuleError),
    ExtensionValidationError(ExtensionValidationError),
    ExtensionVersionAlreadyExistsError(#[expect(unused)] ExtensionVersionAlreadyExistsError),
    #[cynic(fallback)]
    Unknown(String),
}

#[derive(Debug, cynic::QueryVariables)]
pub struct ExtensionPublishVariables {
    pub manifest: extension::VersionedManifest,
    pub wasm_module: Upload,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "ExtensionPublishVariables")]
pub(crate) struct ExtensionPublish {
    #[arguments(manifest: $manifest, wasmModule: $wasm_module)]
    pub(crate) extension_publish: Option<ExtensionPublishPayload>,
}
