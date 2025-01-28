#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    pub name: String,
    pub version: semver::Version,
    pub kind: Kind,
    pub sdk_version: semver::Version,
    pub minimum_gateway_version: semver::Version,
    pub sdl: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Kind {
    FieldResolver(FieldResolver),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FieldResolver {
    pub resolver_directives: Vec<String>,
}
