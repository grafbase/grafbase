use crate::direct_install_executable_path;
use crate::output::report;
use common::consts::USER_AGENT;
use fslock::LockFile;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde::Deserialize;
use std::fs::Permissions;
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;
use tokio::fs::{self, File};
use tokio::io::{self, BufWriter};
use tokio::process::Command;
use tokio::task::{self, JoinError};
use tokio_util::io::StreamReader;

#[derive(Error, Debug)]
pub enum UpgradeError {
    #[error("Could not create a lock for the CLI installation.\nCaused by: {0}")]
    Lock(fslock::Error),

    #[error("Could not release the lock for the CLI installation.\nCaused by:{0}")]
    Unlock(fslock::Error),

    /// returned if the directory cannot be read
    #[error("Could not create path '{0}' for the CLI installation")]
    CreateDir(PathBuf),

    #[error("Encountered an error while downloading grafbase")]
    StartDownload,

    #[error("Encountered an error while downloading grafbase")]
    Download,

    #[error("Could not rename a temporary download file.\nCaused by: {0}")]
    RenameTemporaryFile(io::Error),

    #[error("Could not create a temporary download file.\nCaused by: {0}")]
    CreateTemporaryFile(io::Error),

    #[error("Could not write to a temporary download file.\nCaused by: {0}")]
    WriteTemporaryFile(io::Error),

    #[error("Could not set permissions for the cli executable")]
    SetExecutablePermissions,

    /// returned if a spawned task panics
    #[error(transparent)]
    SpawnedTaskPanic(#[from] JoinError),

    #[error("Encountered an error while determining the latest release version of grafbase")]
    GetLatestReleaseVersion,

    #[error("Encountered an error while determining the installed version of grafbase")]
    GetInstalledVersion,

    #[error("Encountered an error while determining the latest release version")]
    StartGetLatestReleaseVersion,
}

#[derive(Deserialize)]
struct NpmPackageInfo {
    pub version: String,
}

const BINARY_SUFFIX: &str = if cfg!(windows) { ".exe" } else { "" };
const TARGET: &str = env!("TARGET");
const DOWNLOAD_URL_PREFIX: &str = "https://github.com/grafbase/grafbase/releases/download/cli-";
const LATEST_RELEASE_API_URL: &str = "https://registry.npmjs.org/grafbase/latest";
const GRAFBASE_EXECUTABLE_PERMISSIONS: u32 = 0o755;
const GRAFBASE_INSTALL_LOCK_FILE: &str = ".grafbase.install.lock";
const PARTIAL_DOWNLOAD_FILE: &str = ".grafbase.partial";

#[tokio::main]
pub(crate) async fn install_grafbase() -> Result<(), UpgradeError> {
    let direct_install_executable_path = direct_install_executable_path().expect("must exist at this point");
    let lock_file_path = direct_install_executable_path
        .parent()
        .expect("must exist")
        .join(GRAFBASE_INSTALL_LOCK_FILE);
    let mut lock_file = task::spawn_blocking(move || {
        let mut file = LockFile::open(&lock_file_path)?;
        file.lock()?;
        Ok(file)
    })
    .await?
    .map_err(UpgradeError::Lock)?;

    let client = Client::builder().user_agent(USER_AGENT).build().expect("must be valid");

    let latest_version = get_latest_release_version(&client).await?;

    let current_version = get_currently_installed_version().await?;

    if latest_version == current_version {
        report::upgrade_up_to_date(&current_version);
        return Ok(());
    }

    // TODO: use a real progress bar here
    let spinner = ProgressBar::new_spinner()
        .with_message(format!(
            "Downloading grafbase-{TARGET}{BINARY_SUFFIX} {latest_version}..."
        ))
        .with_style(
            ProgressStyle::with_template("{spinner} {wide_msg}")
                .expect("must parse")
                .tick_chars("🕛🕐🕑🕒🕓🕔🕕🕖🕗🕘🕙🕚✨"),
        );

    spinner.enable_steady_tick(Duration::from_millis(100));

    download_grafbase(direct_install_executable_path, client, &latest_version).await?;

    task::spawn_blocking(move || lock_file.unlock())
        .await?
        .map_err(UpgradeError::Unlock)?;

    spinner.finish_with_message(format!("Successfully installed grafbase {latest_version}!"));

    Ok(())
}

async fn download_grafbase(
    direct_install_executable_path: PathBuf,
    client: Client,
    latest_version: &str,
) -> Result<(), UpgradeError> {
    trace!("Installing grafbase…");

    let direct_install_path = direct_install_executable_path.parent().expect("must exist");

    fs::create_dir_all(&direct_install_path)
        .await
        .map_err(|_| UpgradeError::CreateDir(direct_install_path.to_owned()))?;

    let grafbase_temp_binary_path = direct_install_path.join(PARTIAL_DOWNLOAD_FILE);

    let download_url = format!("{DOWNLOAD_URL_PREFIX}{latest_version}/grafbase-{TARGET}{BINARY_SUFFIX}");

    let binary_response = client
        .get(download_url)
        .send()
        .await
        .map_err(|_| UpgradeError::StartDownload)?;

    if !binary_response.status().is_success() {
        return Err(UpgradeError::Download);
    }

    let binary_stream = binary_response
        .bytes_stream()
        .map(|result| result.map_err(|error| io::Error::new(io::ErrorKind::Other, error)));

    let mut temp_binary_file = BufWriter::new(
        File::create(&grafbase_temp_binary_path)
            .await
            .map_err(UpgradeError::CreateTemporaryFile)?,
    );

    let mut reader = StreamReader::new(binary_stream);

    io::copy(&mut reader, &mut temp_binary_file)
        .await
        .map_err(UpgradeError::WriteTemporaryFile)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(
            &grafbase_temp_binary_path,
            Permissions::from_mode(GRAFBASE_EXECUTABLE_PERMISSIONS),
        )
        .await
        .map_err(|_| UpgradeError::SetExecutablePermissions)?;
    }

    // this is done last to prevent leaving the user without a working binary if something errors
    fs::rename(&grafbase_temp_binary_path, &direct_install_executable_path)
        .await
        .map_err(UpgradeError::RenameTemporaryFile)?;

    Ok(())
}

async fn get_latest_release_version(client: &Client) -> Result<String, UpgradeError> {
    let package_response = client
        .get(LATEST_RELEASE_API_URL)
        .send()
        .await
        .map_err(|_| UpgradeError::StartGetLatestReleaseVersion)?;

    if !package_response.status().is_success() {
        return Err(UpgradeError::GetLatestReleaseVersion);
    }

    let package_info: NpmPackageInfo = package_response
        .json()
        .await
        .map_err(|_| UpgradeError::GetLatestReleaseVersion)?;

    Ok(package_info.version.to_owned())
}

async fn get_currently_installed_version() -> Result<String, UpgradeError> {
    Ok(String::from_utf8(
        Command::new(direct_install_executable_path().expect("must exist at this point"))
            .arg("--version")
            .output()
            .await
            .map_err(|_| UpgradeError::GetInstalledVersion)?
            .stdout,
    )
    .map_err(|_| UpgradeError::GetInstalledVersion)?
    .trim_start_matches("Grafbase CLI")
    .trim()
    .to_owned())
}
