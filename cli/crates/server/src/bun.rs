use common::environment::Environment;
use const_format::concatcp;
use futures_util::StreamExt;
use reqwest::Client;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use thiserror::Error;
use tokio::process::Command;
use tokio::task::JoinError;

use crate::atomics::BUN_INSTALLED_FOR_SESSION;

#[derive(Error, Debug, Clone)]
pub enum CommandError {
    #[error("working directory '{0}' not found")]
    WorkingDirectoryNotFound(PathBuf),

    #[error("working directory '{0}' cannot be read.\nCaused by: {1}")]
    WorkingDirectoryCannotBeRead(PathBuf, Arc<std::io::Error>),

    /// returned if any of the bun commands cannot be spawned
    #[error("bun encountered an error: {0}")]
    Spawn(Arc<std::io::Error>),

    /// returned if any of the bun commands exits unsuccessfully
    #[error("bun failed with output:\n{0}")]
    OutputError(String),
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
}

impl From<JoinError> for BunError {
    fn from(value: JoinError) -> Self {
        Self::SpawnedTaskPanic(Arc::new(value))
    }
}

// TODO: add windows once supported in Bun

const BUN_VERSION: &str = "1.0.29";

#[cfg(target_arch = "aarch64")]
const ARCH: &str = "aarch64";

#[cfg(target_arch = "x86_64")]
const ARCH: &str = "x64";

#[cfg(target_os = "macos")]
const OS: &str = "darwin";

#[cfg(target_os = "linux")]
const OS: &str = "linux";

const DOWNLOAD_URL: &str = concatcp!(
    "https://github.com/oven-sh/bun/releases/download/bun-v",
    BUN_VERSION,
    "/bun-",
    OS,
    "-",
    ARCH,
    ".zip"
);

async fn run_command<P: AsRef<Path>>(
    program_path: &Path,
    arguments: &[&str],
    current_directory: P,
) -> Result<Option<Vec<u8>>, CommandError> {
    let current_directory = current_directory.as_ref();
    match current_directory.try_exists() {
        Ok(true) => Ok(()),
        Ok(false) => Err(CommandError::WorkingDirectoryNotFound(current_directory.to_owned())),
        Err(err) => Err(CommandError::WorkingDirectoryCannotBeRead(
            current_directory.to_owned(),
            Arc::new(err),
        )),
    }?;

    let mut command = Command::new(program_path);
    command
        .args(arguments)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(current_directory);

    trace!("Spawning {command:?}");
    let command = command.spawn().map_err(Arc::new).map_err(CommandError::Spawn)?;

    let output = command
        .wait_with_output()
        .await
        .map_err(Arc::new)
        .map_err(CommandError::Spawn)?;

    if output.status.success() {
        Ok(Some(output.stdout).filter(Vec::is_empty))
    } else {
        Err(CommandError::OutputError(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ))
    }
}

const BUN_EXECUTABLE_PERMISSIONS: u32 = 0o755;
const BUN_INSTALL_LOCK_FILE: &str = ".bun.install.lock";

pub(crate) async fn install_bun() -> Result<(), BunError> {
    if BUN_INSTALLED_FOR_SESSION.load(Ordering::Acquire) {
        return Ok(());
    }

    let environment = Environment::get();
    let lock_file_path = environment.user_dot_grafbase_path.join(BUN_INSTALL_LOCK_FILE);
    let mut lock_file = tokio::task::spawn_blocking(move || {
        let mut file = fslock::LockFile::open(&lock_file_path).map_err(Arc::new)?;
        file.lock().map_err(Arc::new)?;
        Ok(file)
    })
    .await?
    .map_err(BunError::Lock)?;

    if let Some(installed_bun_version) = installed_bun_version(&environment.bun_executable_path).await {
        trace!("Installed bun version (grafbase): {installed_bun_version}");
        if installed_bun_version == BUN_VERSION {
            trace!("bun of the desired version already installed, skipping…");
            return Ok(());
        }
    }

    // if the user happens to have the bun binary installed globally with the exact version we require (not >= but ==, to avoid untested behavior)
    // we hard-link it instead of downloading.
    if let Ok(system_bun_path) = which::which("bun") {
        if let Some(installed_bun_version) = installed_bun_version(&system_bun_path).await {
            trace!("Installed bun version (system): {installed_bun_version}");
            if installed_bun_version == BUN_VERSION {
                trace!("bun of the desired version already installed system-wide, hard linking…");
                tokio::fs::create_dir_all(&environment.bun_installation_path)
                    .await
                    .map_err(|_| BunError::CreateDir(environment.bun_installation_path.clone()))?;
                if environment.bun_executable_path.exists() {
                    tokio::fs::remove_file(&environment.bun_executable_path).await.unwrap();
                }
                tokio::fs::hard_link(system_bun_path, &environment.bun_executable_path)
                    .await
                    .unwrap();
                return Ok(());
            }
        }
    }

    download_bun(environment).await?;

    tokio::task::spawn_blocking(move || lock_file.unlock())
        .await?
        .map_err(Arc::new)
        .map_err(BunError::Unlock)?;

    BUN_INSTALLED_FOR_SESSION.store(true, Ordering::Release);

    Ok(())
}

async fn download_bun(environment: &Environment) -> Result<(), BunError> {
    trace!("Installing bun…");
    tokio::fs::create_dir_all(&environment.bun_installation_path)
        .await
        .map_err(|_| BunError::CreateDir(environment.bun_installation_path.clone()))?;
    if environment.bun_executable_path.exists() {
        tokio::fs::remove_file(&environment.bun_executable_path).await.unwrap();
    }
    let zip_response = Client::new().get(DOWNLOAD_URL).send().await.unwrap();
    if !zip_response.status().is_success() {
        // return Err(BackendError::DownloadRepoArchive(org_and_repo.to_owned()));
    }
    let zip_stream = zip_response
        .bytes_stream()
        .map(|result| result.map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error)));
    let mut zip_reader = tokio_util::io::StreamReader::new(zip_stream);
    let (decompressed_file, decompressed_file_path) = tempfile::NamedTempFile::new()
        .unwrap()
        // .map_err(BackendError::CouldNotCreateTemporaryFile)?
        .into_parts();
    let mut decompressed_file: tokio::fs::File = decompressed_file.into();
    tokio::io::copy(&mut zip_reader, &mut decompressed_file).await.unwrap();
    decompressed_file.sync_all().await.unwrap();
    tokio::task::spawn_blocking(move || {
        let environment = Environment::get();

        let decompressed_file = std::fs::File::open(&decompressed_file_path).unwrap();
        // .map_err(BackendError::CouldNotCreateTemporaryFile)?
        let mut archive = zip::ZipArchive::new(decompressed_file).unwrap();

        // the archive contains a directory which has a single file - the bun binary
        let mut binary = archive.by_index(1).unwrap();

        let mut outfile = std::fs::File::create(&environment.bun_executable_path).unwrap();
        std::io::copy(&mut binary, &mut outfile).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            std::fs::set_permissions(
                &environment.bun_executable_path,
                std::fs::Permissions::from_mode(BUN_EXECUTABLE_PERMISSIONS),
            )
            .unwrap();
        }

        Ok::<_, BunError>(())
    })
    .await
    .expect("must succeed")
    .unwrap();
    drop(decompressed_file);
    Ok(())
}

async fn installed_bun_version(bun_executable_path: &Path) -> Option<String> {
    let bun_arguments = &["--version"];
    let output_bytes = run_command(
        bun_executable_path,
        bun_arguments,
        bun_executable_path.parent().expect("must exist"),
    )
    .await
    .ok()??;
    Some(String::from_utf8(output_bytes).ok()?.trim().to_owned())
}
