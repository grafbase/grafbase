#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    pub name: String,
    pub version: semver::Version,
    pub kind: Kind,
    pub sdk_version: semver::Version,
    pub minimum_gateway_version: semver::Version,
    pub sdl: Option<String>,
}

impl Manifest {
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
