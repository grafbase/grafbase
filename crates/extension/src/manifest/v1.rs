use crate::Id;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    pub id: Id,
    pub kind: Kind,
    pub sdk_version: semver::Version,
    pub minimum_gateway_version: semver::Version,
    pub sdl: Option<String>,
}

impl Manifest {
    pub fn name(&self) -> &str {
        &self.id.name
    }

    pub fn version(&self) -> &semver::Version {
        &self.id.version
    }

    pub fn into_versioned(self) -> super::VersionedManifest {
        super::VersionedManifest::V1(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Kind {
    FieldResolver(FieldResolver),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FieldResolver {
    pub resolver_directives: Vec<String>,
}
