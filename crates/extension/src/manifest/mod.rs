mod v1;

// importing latest version of the manifest.
pub use v1::*;

/// Suitable for long-term storage as backwards-compatibility will be maintained.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "version", rename_all = "lowercase")]
pub enum VersionedManifest {
    V1(v1::Manifest),
}

impl VersionedManifest {
    pub fn into_latest(self) -> Manifest {
        match self {
            VersionedManifest::V1(v1) => v1,
        }
    }
}
