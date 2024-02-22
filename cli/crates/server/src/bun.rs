use common::environment::Environment;

use itertools::Itertools;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use thiserror::Error;
use tokio::process::Command;
use tokio::task::JoinError;

#[derive(Error, Debug, Clone)]
pub enum NpmCommandError {
    #[error("working directory '{0}' not found")]
    WorkingDirectoryNotFound(PathBuf),
    #[error("working directory '{0}' cannot be read.\nCaused by: {1}")]
    WorkingDirectoryCannotBeRead(PathBuf, Arc<std::io::Error>),
    /// returned if npm cannot be found
    #[error("could not find npm: {0}")]
    NotFound(which::Error),

    /// returned if any of the npm commands exits unsuccessfully
    #[error("npm encountered an error: {0}")]
    CommandError(Arc<std::io::Error>),

    /// returned if any of the npm commands exits unsuccessfully
    #[error("npm failed with output:\n{0}")]
    OutputError(String),
    /// returned if any of the bun commands exits unsuccessfully
    #[error("bun encountered an error: {0}")]
    BunCommandError(String),

    /// returned if any of the bun commands exits unsuccessfully
    #[error("bun failed with output:\n{0}")]
    BunOutputError(String),
}
// this has to be clone for use in the config manager
#[derive(Error, Debug, Clone)]
pub enum BunError {
    #[error("Could not create a lock for the bun installation: {0}")]
    Lock(Arc<fslock::Error>),

    #[error("Could not release the lock for the bun installation: {0}")]
    Unlock(Arc<fslock::Error>),

    /// returned if the directory cannot be read
    #[error("could not create path '{0}' for the bun installation")]
    CreateDir(PathBuf),

    /// returned if a spawned task panics
    #[error("{0}")]
    SpawnedTaskPanic(Arc<JoinError>),

    /// returned if any of the package manager commands ran during resolver build exits unsuccessfully
    #[error("command error: {0}")]
    BunInstallPackageManagerCommandError(#[from] NpmCommandError),
}

impl From<JoinError> for BunError {
    fn from(value: JoinError) -> Self {
        Self::SpawnedTaskPanic(Arc::new(value))
    }
}

const BUN_VERSION: &str = "1.0.28";

async fn run_command<P: AsRef<Path>>(
    arguments: &[&str],
    current_directory: P,
) -> Result<Option<Vec<u8>>, NpmCommandError> {
    let command_string = format!("npm {}", arguments.iter().format(" "));
    let current_directory = current_directory.as_ref();
    match current_directory.try_exists() {
        Ok(true) => Ok(()),
        Ok(false) => Err(NpmCommandError::WorkingDirectoryNotFound(current_directory.to_owned())),
        Err(err) => Err(NpmCommandError::WorkingDirectoryCannotBeRead(
            current_directory.to_owned(),
            Arc::new(err),
        )),
    }?;
    trace!("running '{command_string}'");

    // Use `which` to work-around weird path search issues on Windows.
    // See https://github.com/rust-lang/rust/issues/37519.
    let program_path = which::which("npm").map_err(NpmCommandError::NotFound)?;

    let mut command = Command::new(program_path);
    command
        .args(arguments)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(current_directory);

    trace!("Spawning {command:?}");
    let command = command
        .spawn()
        .map_err(Arc::new)
        .map_err(NpmCommandError::CommandError)?;

    let output = command
        .wait_with_output()
        .await
        .map_err(Arc::new)
        .map_err(NpmCommandError::CommandError)?;

    if output.status.success() {
        trace!("'{command_string}' succeeded");
        Ok(Some(output.stdout).filter(|output| !output.is_empty()))
    } else {
        trace!("'{command_string}' failed");
        Err(NpmCommandError::OutputError(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ))
    }
}

async fn installed_bun_version(bun_installation_path: impl AsRef<Path>) -> Option<String> {
    let bun_installation_path = bun_installation_path.as_ref();
    let bun_arguments = &[
        "exec",
        "--no",
        "--prefix",
        bun_installation_path.to_str().expect("must be valid"),
        "--",
        "bun",
        "--version",
    ];
    let output_bytes = run_command(bun_arguments, bun_installation_path).await.ok()??;
    Some(String::from_utf8(output_bytes).ok()?.trim().to_owned())
}

pub(crate) async fn install_bun(environment: &Environment) -> Result<(), BunError> {
    let lock_file_path = environment.user_dot_grafbase_path.join(".bun.install.lock");
    let mut lock_file = tokio::task::spawn_blocking(move || {
        let mut file = fslock::LockFile::open(&lock_file_path).map_err(Arc::new)?;
        file.lock().map_err(Arc::new)?;
        Ok(file)
    })
    .await?
    .map_err(BunError::Lock)?;

    if let Some(installed_bun_version) = installed_bun_version(&environment.bun_installation_path).await {
        trace!("Installed bun version: {installed_bun_version}");
        if installed_bun_version == BUN_VERSION {
            trace!("bun of the desired version already installed, skipping…");
            return Ok(());
        }
    }

    let bun_installation_path_str = environment.bun_installation_path.to_str().expect("must be valid");

    trace!("Installing bun…");
    tokio::fs::create_dir_all(&environment.bun_installation_path)
        .await
        .map_err(|_| BunError::CreateDir(environment.bun_installation_path.clone()))?;
    // Install bun once and for all.
    run_command(
        &[
            "add",
            "--save-dev",
            &format!("bun@{BUN_VERSION}"),
            "--prefix",
            bun_installation_path_str,
        ],
        bun_installation_path_str,
    )
    .await?;

    tokio::task::spawn_blocking(move || lock_file.unlock())
        .await?
        .map_err(Arc::new)
        .map_err(BunError::Unlock)?;

    Ok(())
}
