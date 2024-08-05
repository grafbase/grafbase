use std::path::PathBuf;

/// GraphQL WASI component configuration.
#[derive(Clone, Default, Debug, serde::Deserialize)]
pub struct HooksWasiConfig {
    pub location: PathBuf,
    #[serde(default)]
    pub networking: bool,
    #[serde(default)]
    pub environment_variables: bool,
    #[serde(default)]
    pub stdout: bool,
    #[serde(default)]
    pub stderr: bool,
    #[serde(default)]
    pub preopened_directories: Vec<PreopenedDirectory>,
}

/// Configuration for allowing access to a certain directory from a WASI guest
#[derive(Clone, Debug, serde::Deserialize)]
pub struct PreopenedDirectory {
    pub host_path: PathBuf,
    pub guest_path: String,
    pub read_permission: bool,
    pub write_permission: bool,
}
