mod v1;

use semver::Version;
// importing latest version of the manifest.
pub use v1::*;

/// Suitable for long-term storage as backwards-compatibility will be maintained.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "manifest", rename_all = "lowercase")]
pub enum VersionedManifest {
    V1(v1::Manifest),
}

impl VersionedManifest {
    pub fn minimum_gateway_version(&self) -> &Version {
        match self {
            VersionedManifest::V1(v1) => &v1.minimum_gateway_version,
        }
    }

    pub fn sdk_version(&self) -> &Version {
        match self {
            VersionedManifest::V1(v1) => &v1.sdk_version,
        }
    }
}

impl VersionedManifest {
    pub fn into_latest(self) -> Manifest {
        match self {
            VersionedManifest::V1(v1) => v1,
        }
    }
}
