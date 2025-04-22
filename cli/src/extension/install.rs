use crate::{backend::api, output::report};
use extension::lockfile;
use futures::stream::FuturesUnordered;
use gateway_config::Config;
use std::{borrow::Cow, io, path::Path};
use tokio::{fs, io::AsyncWriteExt as _};
use tokio_stream::StreamExt as _;
use url::Url;

use super::EXTENSION_WASM_MODULE_FILE_NAME;

pub const PUBLIC_EXTENSION_REGISTRY_URL: &str = "https://extensions.grafbase.com";

pub(crate) async fn execute(config: &Config) -> anyhow::Result<()> {
    if let Some(new_lockfile) = handle_lockfile(config).await? {
        download_extensions(new_lockfile).await?;
    }
    Ok(())
}

async fn download_extensions(new_lockfile: lockfile::Lockfile) -> anyhow::Result<()> {
    let extensions_directory = Path::new(extension_catalog::EXTENSION_DIR_NAME);
    let http_client = reqwest::Client::new();

    let base_registry_url: url::Url = std::env::var("EXTENSION_REGISTRY_URL")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| PUBLIC_EXTENSION_REGISTRY_URL.parse().unwrap());

    fs::create_dir_all(extensions_directory).await.map_err(|err| {
        anyhow::anyhow!(
            "Failed to create extensions directory at {}. Cause: {err}",
            extensions_directory.display()
        )
    })?;

    let mut futures = FuturesUnordered::new();

    report::extension_install_start();

    let progress_bar = indicatif::ProgressBar::new(new_lockfile.extensions.len() as u64);

    for extension in new_lockfile.extensions {
        futures.push(download_extension_from_registry(
            &http_client,
            extensions_directory,
            extension.name,
            extension.version,
            &base_registry_url,
        ));
    }

    while let Some(()) = futures.try_next().await? {
        progress_bar.inc(1);
    }

    progress_bar.finish_with_message("All extensions downloaded in ....");

    Ok(())
}

/// Returns the new up to date lockfile.
async fn handle_lockfile(config: &Config) -> anyhow::Result<Option<lockfile::Lockfile>> {
    let mut has_updated = false;
    let lockfile_path = Path::new(extension::lockfile::EXTENSION_LOCKFILE_NAME);
    let lockfile_str = match fs::read_to_string(lockfile_path).await {
        Ok(str) => Some(str),
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound => None,
            _ => anyhow::bail!("Failed to read lockfile at {}. Cause: {err}", lockfile_path.display()),
        },
    };

    let lockfile: lockfile::Lockfile = if let Some(lockfile_str) = lockfile_str {
        let extension::lockfile::VersionedLockfile::V1(lockfile) = toml::from_str(&lockfile_str)
            .map_err(|err| anyhow::anyhow!("Failed to parse lockfile at {}. Cause: {err}", lockfile_path.display()))?;

        lockfile
    } else {
        lockfile::Lockfile::default()
    };

    let mut new_version_requirements: Vec<(String, semver::VersionReq)> = Vec::new();

    let mut extensions_from_config = config.extensions.clone();

    // We ignore extensions that have explicitly a path.
    extensions_from_config.retain(|_, ext| ext.path().is_none());

    if extensions_from_config.is_empty() {
        report::no_extension_defined_in_config();
        return Ok(None);
    }

    let mut new_lockfile = lockfile::Lockfile::default();

    for ext in lockfile.extensions {
        match extensions_from_config.remove(&ext.name) {
            Some(requirement) => {
                if requirement.version().matches(&ext.version) {
                    new_lockfile.extensions.push(ext);
                } else {
                    new_version_requirements.push((ext.name, requirement.version().clone()));
                }
            }
            None => {
                has_updated = true;
            }
        }
    }

    for (name, ext) in extensions_from_config {
        new_version_requirements.push((name, ext.version().clone()));
    }

    if !new_version_requirements.is_empty() {
        has_updated = true;

        let matches = api::extension_versions_by_version_requirement::extension_versions_by_version_requirement(
            new_version_requirements
                .iter()
                .map(|(name, version)| (name.clone(), version.clone())),
        )
        .await?;

        for (i, m) in matches.into_iter().enumerate() {
            match m {
                api::extension_versions_by_version_requirement::ExtensionVersionMatch::Match { name, version } => {
                    new_lockfile.extensions.push(lockfile::Extension { name, version });
                }
                api::extension_versions_by_version_requirement::ExtensionVersionMatch::ExtensionDoesNotExist => {
                    super::update::handle_extension_does_not_exist(&new_version_requirements[i].0);
                }
                api::extension_versions_by_version_requirement::ExtensionVersionMatch::ExtensionVersionDoesNotExist => {
                    let (name, version) = &new_version_requirements[i];
                    super::update::handle_extension_version_does_not_exist(name, version);
                }
            }
        }
    }

    if has_updated {
        let new_lockfile_str = toml::ser::to_string_pretty(&lockfile::VersionedLockfile::V1(new_lockfile.clone()))
            .map_err(|err| anyhow::anyhow!("Failed to serialize new lockfile: {err}"))?;
        fs::write(lockfile_path, new_lockfile_str.as_bytes()).await?;
    }

    Ok(Some(new_lockfile))
}

async fn download_extension_from_registry(
    http_client: &reqwest::Client,
    extensions_dir: &Path,
    extension_name: String,
    version: semver::Version,
    registry_base_url: &Url,
) -> Result<(), Report> {
    let files = ["manifest.json", EXTENSION_WASM_MODULE_FILE_NAME];

    let [manifest_fut, wasm_fut] = files.map(|file_name| {
        let mut url = registry_base_url.clone();
        url.set_path(&format!("/extensions/{extension_name}/{version}/{file_name}"));
        let dir_path = extensions_dir.join(&extension_name).join(version.to_string());
        let file_path = dir_path.join(file_name);

        async move {
            if let Ok(true) = fs::try_exists(&file_path).await {
                return Ok(());
            }

            let response = http_client
                .get(url.clone())
                .send()
                .await
                .map_err(|err| Report::http(err, &url))?;

            if !response.status().is_success() {
                return Err(Report::http_status(response.status(), &url));
            }

            fs::create_dir_all(&dir_path)
                .await
                .map_err(|_| Report::create_dir(&dir_path))?;

            // Create the output file
            let mut file = fs::File::create(&file_path)
                .await
                .map_err(|err| Report::create_file(&file_path, err))?;

            // Stream the body bytes to the file
            let mut stream = response.bytes_stream();

            while let Some(chunk) = stream.try_next().await.map_err(Report::body_stream)? {
                file.write_all(&chunk).await.map_err(Report::write)?;
            }

            Ok(())
        }
    });

    tokio::try_join!(manifest_fut, wasm_fut).map(|_| ())
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
struct Report(Cow<'static, str>);

impl Report {
    fn create_dir(path: &Path) -> Self {
        Report(Cow::Owned(format!("Failed to create directory: {}", path.display())))
    }

    fn create_file(path: &Path, err: io::Error) -> Self {
        Report(Cow::Owned(format!(
            "Failed to create file: {} ({})",
            path.display(),
            err
        )))
    }

    fn write(err: io::Error) -> Self {
        Report(err.to_string().into())
    }

    fn body_stream(err: reqwest::Error) -> Self {
        Report(Cow::Owned(format!("Failed to read response body: {err}")))
    }

    fn http(err: reqwest::Error, url: &Url) -> Self {
        Report(Cow::Owned(format!(
            "HTTP error downloading extension from {url}: {err}",
        )))
    }

    fn http_status(status: reqwest::StatusCode, url: &Url) -> Self {
        Report(Cow::Owned(format!(
            "HTTP error downloading extension from {url}: {status}",
        )))
    }
}
