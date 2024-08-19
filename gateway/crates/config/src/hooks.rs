use std::path::PathBuf;

/// GraphQL WASI component configuration.
#[derive(Clone, Default, Debug, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct HooksWasiConfig {
    pub location: PathBuf,
    pub networking: bool,
    pub environment_variables: bool,
    pub stdout: bool,
    pub stderr: bool,
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
