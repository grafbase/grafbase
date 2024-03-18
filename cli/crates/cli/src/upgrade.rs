use common::consts::USER_AGENT;
use common::environment::Environment;
use const_format::concatcp;
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
    CreateDir(&'static PathBuf),

    // TODO: add specific connection error
    #[error("Encountered an error while downloading grafbase")]
    Download,

    #[error("Could not rename a temporary download file.\nCaused by: {0}")]
    RenameTemporaryFile(io::Error),

    #[error("Could not create a temporary download file.\nCaused by: {0}")]
    CreateTemporaryFile(io::Error),

    #[error("Could not set permissions for the cli executable")]
    SetExecutablePermissions,

    /// returned if a spawned task panics
    #[error(transparent)]
    SpawnedTaskPanic(#[from] JoinError),

    // TODO: add specfic connection error
    #[error("Encountered an error while determining the latest release version")]
    GetLatestReleaseVersion,

    #[error("The locally installed version of grafbase is already up to date")]
    UpToDate,
}

#[derive(Deserialize)]
struct NpmPackageInfo {
    pub version: String,
}

const BINARY_SUFFIX: &str = if cfg!(windows) { ".exe" } else { "" };
const TARGET: &str = env!("TARGET");
const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const LATEST_RELEASE_API_URL: &str = "https://registry.npmjs.org/grafbase/latest";
const GRAFBASE_EXECUTABLE_PERMISSIONS: u32 = 0o755;
const GRAFBASE_INSTALL_LOCK_FILE: &str = ".grafbase.install.lock";
const PARTIAL_DOWNLOAD_FILE: &str = ".grafbase.partial";
const EXECUTABLE_NAME: &str = concatcp!("grafbase", BINARY_SUFFIX);

#[tokio::main]
pub(crate) async fn install_grafbase() -> Result<(), UpgradeError> {
    let environment = Environment::get();
    let lock_file_path = environment.user_dot_grafbase_path.join(GRAFBASE_INSTALL_LOCK_FILE);
    let mut lock_file = task::spawn_blocking(move || {
        let mut file = LockFile::open(&lock_file_path)?;
        file.lock()?;
        Ok(file)
    })
    .await?
    .map_err(UpgradeError::Lock)?;

    let client = Client::builder().user_agent(USER_AGENT).build().expect("must be valid");

    let latest_version = get_latest_release_version(&client).await?;

    // TODO: consider getting this from the binary to prevent multiple simultainious runs of uppgrade
    // from downloading the same binary after the lock is released

    if latest_version == CARGO_PKG_VERSION {
        return Err(UpgradeError::UpToDate);
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

    download_grafbase(environment, client, &latest_version).await?;

    task::spawn_blocking(move || lock_file.unlock())
        .await?
        .map_err(UpgradeError::Unlock)?;

    spinner.finish_with_message(format!("Successfully installed grafbase {latest_version}!"));

    Ok(())
}

async fn download_grafbase(
    environment: &'static Environment,
    client: Client,
    latest_version: &str,
) -> Result<(), UpgradeError> {
    trace!("Installing grafbase…");
    fs::create_dir_all(&environment.grafbase_installation_path)
        .await
        .map_err(|_| UpgradeError::CreateDir(&environment.grafbase_installation_path))?;

    let grafbase_binary_path = environment.grafbase_installation_path.join(EXECUTABLE_NAME);
    let grafbase_temp_binary_path = environment.grafbase_installation_path.join(PARTIAL_DOWNLOAD_FILE);

    let download_url = format!(
        "https://github.com/grafbase/grafbase/releases/download/cli-{}/grafbase-{}{}",
        latest_version, TARGET, BINARY_SUFFIX
    );

    let binary_response = client
        .get(download_url)
        .send()
        .await
        .map_err(|_| UpgradeError::Download)?;

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
        .map_err(|_| UpgradeError::Download)?;

    fs::rename(&grafbase_temp_binary_path, &grafbase_binary_path)
        .await
        .map_err(UpgradeError::RenameTemporaryFile)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(
            &grafbase_binary_path,
            Permissions::from_mode(GRAFBASE_EXECUTABLE_PERMISSIONS),
        )
        .await
        .map_err(|_| UpgradeError::SetExecutablePermissions)?;
    }

    Ok(())
}

async fn get_latest_release_version(client: &Client) -> Result<String, UpgradeError> {
    let package_info: NpmPackageInfo = client
        .get(LATEST_RELEASE_API_URL)
        .send()
        .await
        .inspect_err(|error| {
            dbg!(error);
        })
        .map_err(|_| UpgradeError::GetLatestReleaseVersion)?
        .json()
        .await
        .inspect_err(|error| {
            dbg!(error);
        })
        .map_err(|_| UpgradeError::GetLatestReleaseVersion)?;
    Ok(package_info.version.to_owned())
}
