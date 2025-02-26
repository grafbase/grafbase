#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct Lockfile {
    pub extensions: Vec<Extension>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Extension {
    pub name: String,
    pub version: semver::Version,
}
