use crate::Id;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    pub id: Id,
    pub kind: Kind,
    pub sdk_version: semver::Version,
    pub minimum_gateway_version: semver::Version,
    #[serde(skip_serializing_if = "Option::is_none")]
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

    pub fn is_resolver(&self) -> bool {
        matches!(self.kind, Kind::FieldResolver(_))
    }

    pub fn is_authenticator(&self) -> bool {
        matches!(self.kind, Kind::Authenticator)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Kind {
    FieldResolver(FieldResolver),
    Authenticator,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FieldResolver {
    pub resolver_directives: Vec<String>,
}
