use std::path::PathBuf;

/// Configuration for the GraphQL WASI component hooks.
#[derive(Clone, Default, Debug, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct HooksWasiConfig {
    /// The location of the WASI component.
    pub location: PathBuf,
    /// Indicates if networking is enabled for the WASI component.
    pub networking: bool,
    /// Indicates if environment variables should be available to the WASI component.
    pub environment_variables: bool,
    /// Indicates if standard output should be available to the WASI component.
    pub stdout: bool,
    /// Indicates if standard error should be available to the WASI component.
    pub stderr: bool,
    /// A list of directories that are preopened for the WASI component.
    pub preopened_directories: Vec<PreopenedDirectory>,
    /// The maximum number of concurrent instances of the WASI component. Defaults to four times the number of CPUs.
    pub max_pool_size: Option<usize>,
}

/// Configuration for a directory that is preopened for the WASI component.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct PreopenedDirectory {
    /// The path on the host system that is preopened.
    pub host_path: PathBuf,
    /// The corresponding path in the guest environment.
    pub guest_path: String,
    /// Indicates if read access is permitted for this directory.
    pub read_permission: bool,
    /// Indicates if write access is permitted for this directory.
    pub write_permission: bool,
}
