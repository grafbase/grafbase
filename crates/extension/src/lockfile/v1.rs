#[derive(serde::Deserialize, serde::Serialize, Default, Clone)]
pub struct Lockfile {
    pub extensions: Vec<Extension>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Extension {
    pub name: String,
    pub version: semver::Version,
}
