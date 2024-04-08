use common::consts::USER_AGENT;
use common::environment::Environment;
use const_format::concatcp;
use futures_util::StreamExt;
use hyper::header;
use reqwest::Client;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use thiserror::Error;
use tokio::process::Command;
use tokio::task::JoinError;
use zip::result::ZipError;

use crate::atomics::BUN_INSTALLED_FOR_SESSION;

#[derive(Error, Debug, Clone)]
pub enum CommandError {
    #[error("working directory '{0}' not found")]
    WorkingDirectoryNotFound(PathBuf),

    #[error("working directory '{0}' cannot be read.\nCaused by: {1}")]
    WorkingDirectoryCannotBeRead(PathBuf, Arc<io::Error>),

    /// returned if any of the bun commands cannot be spawned
    #[error("bun encountered an error: {0}")]
    Spawn(Arc<io::Error>),

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

    #[error("could not remove a stale version of bun.\nCaused by: {0}")]
    RemoveStaleBunVersion(Arc<io::Error>),

    #[error("could not hard-link the system bun version.\nCaused by: {0}")]
    HardLink(Arc<io::Error>),

    #[error("encountered an error while downloading bun")]
    DownloadBun,

    #[error("could not create a temporary file")]
    CreateTemporaryFile,

    #[error("could not extract the bun archive\nCaused by: {0}")]
    ExtractBunArchive(Arc<io::Error>),

    #[error("could not set permissions for the bun executable")]
    SetBunExecutablePermissions,
}

impl From<JoinError> for BunError {
    fn from(value: JoinError) -> Self {
        Self::SpawnedTaskPanic(Arc::new(value))
    }
}

impl From<ZipError> for BunError {
    fn from(error: ZipError) -> Self {
        Self::ExtractBunArchive(Arc::new(error.into()))
    }
}

const BUN_VERSION: &str = "1.1.2";

#[cfg(target_arch = "aarch64")]
const ARCH: &str = "aarch64";

#[cfg(target_arch = "x86_64")]
const ARCH: &str = "x64";

#[cfg(target_os = "macos")]
const OS: &str = "darwin";

#[cfg(target_os = "linux")]
const OS: &str = "linux";

#[cfg(target_os = "windows")]
const OS: &str = "windows";

const BUN_DOWNLOAD_URL: &str = concatcp!(
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
        Ok(Some(output.stdout).filter(|output| !output.is_empty()))
    } else {
        Err(CommandError::OutputError(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ))
    }
}

#[cfg(unix)]
const BUN_EXECUTABLE_PERMISSIONS: u32 = 0o755;

const BUN_INSTALL_LOCK_FILE: &str = ".bun.install.lock";

// the archive contains a directory which has a single file - the bun binary
// this index appears to differ between unix and windows
const BUN_EXECUTABLE_ARCHIVE_FILE_INDEX: usize = if cfg!(unix) { 1 } else { 0 };

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

    if let Some(installed_bun_version_string) = installed_bun_version(&environment.bun_executable_path).await {
        trace!("Installed bun version (grafbase): {installed_bun_version_string}");
        if installed_bun_version_string == BUN_VERSION {
            trace!("bun of the desired version already installed, skipping…");
            BUN_INSTALLED_FOR_SESSION.store(true, Ordering::Release);
            return Ok(());
        }
    }

    // if the user happens to have the bun binary installed globally with the exact version we require (not >= but ==, to avoid untested behavior)
    // we hard-link it instead of downloading.
    if let Ok(system_bun_path) = which::which("bun") {
        if let Some(installed_bun_version_string) = installed_bun_version(&system_bun_path).await {
            trace!("Installed bun version (system): {installed_bun_version_string}");
            if installed_bun_version_string == BUN_VERSION {
                trace!("bun of the desired version already installed system-wide, hard linking…");
                tokio::fs::create_dir_all(&environment.bun_installation_path)
                    .await
                    .map_err(|_| BunError::CreateDir(environment.bun_installation_path.clone()))?;
                if environment.bun_executable_path.exists() {
                    tokio::fs::remove_file(&environment.bun_executable_path)
                        .await
                        .map_err(Arc::new)
                        .map_err(BunError::RemoveStaleBunVersion)?;
                }
                // if we can't hard link the system version (can happen due to being on a different volume)
                // continue to a normal download
                if tokio::fs::hard_link(system_bun_path, &environment.bun_executable_path)
                    .await
                    .is_ok()
                {
                    BUN_INSTALLED_FOR_SESSION.store(true, Ordering::Release);
                    return Ok(());
                }

                trace!("could not hard-link the system version of bun, continuing to download…");
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
        tokio::fs::remove_file(&environment.bun_executable_path)
            .await
            .map_err(Arc::new)
            .map_err(BunError::RemoveStaleBunVersion)?;
    }

    let zip_response = Client::new()
        .get(BUN_DOWNLOAD_URL)
        .header(header::USER_AGENT, USER_AGENT)
        .send()
        .await
        .map_err(|_| BunError::DownloadBun)?;

    if !zip_response.status().is_success() {
        return Err(BunError::DownloadBun);
    }

    let zip_stream = zip_response
        .bytes_stream()
        .map(|result| result.map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error)));

    let mut zip_reader = tokio_util::io::StreamReader::new(zip_stream);

    let (decompressed_file, decompressed_file_path) = tempfile::NamedTempFile::new()
        .map_err(|_| BunError::CreateTemporaryFile)?
        .into_parts();

    let mut decompressed_file: tokio::fs::File = decompressed_file.into();

    tokio::io::copy(&mut zip_reader, &mut decompressed_file)
        .await
        .map_err(|_| BunError::DownloadBun)?;

    decompressed_file.sync_all().await.map_err(|_| BunError::DownloadBun)?;

    tokio::task::spawn_blocking(move || {
        let environment = Environment::get();

        let decompressed_file = std::fs::File::open(&decompressed_file_path)
            .map_err(Arc::new)
            .map_err(BunError::ExtractBunArchive)?;

        let mut archive = zip::ZipArchive::new(decompressed_file)?;

        let mut binary = archive.by_index(BUN_EXECUTABLE_ARCHIVE_FILE_INDEX)?;

        let mut outfile = std::fs::File::create(&environment.bun_executable_path)
            .map_err(Arc::new)
            .map_err(BunError::ExtractBunArchive)?;

        std::io::copy(&mut binary, &mut outfile)
            .map_err(Arc::new)
            .map_err(BunError::ExtractBunArchive)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            std::fs::set_permissions(
                &environment.bun_executable_path,
                std::fs::Permissions::from_mode(BUN_EXECUTABLE_PERMISSIONS),
            )
            .map_err(|_| BunError::SetBunExecutablePermissions)?;
        }

        Ok::<_, BunError>(())
    })
    .await
    .expect("must succeed")?;

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
