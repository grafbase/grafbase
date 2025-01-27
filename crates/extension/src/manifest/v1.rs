#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    pub name: String,
    pub version: semver::Version,
    pub kind: Kind,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Kind {
    FieldResolver(FieldResolver),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FieldResolver {
    resolver_directives: Vec<String>,
}
