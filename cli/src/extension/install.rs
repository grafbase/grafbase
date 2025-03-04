use crate::{backend::api, cli_input::ExtensionInstallCommand, output::report};
use extension::lockfile;
use extension_catalog::PUBLIC_EXTENSION_REGISTRY_URL;
use futures::stream::FuturesUnordered;
use std::path::Path;
use tokio::fs;
use tokio_stream::StreamExt as _;

#[tokio::main]
pub(super) async fn execute(cmd: ExtensionInstallCommand) -> anyhow::Result<()> {
    let new_lockfile = handle_lockfile(&cmd.config).await?;
    download_extensions(new_lockfile).await
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

    report::extension_install_start(extensions_directory);

    let progress_bar = indicatif::ProgressBar::new(new_lockfile.extensions.len() as u64);

    for extension in new_lockfile.extensions {
        futures.push(extension_catalog::download_extension_from_registry(
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
async fn handle_lockfile(config_path: &Path) -> anyhow::Result<lockfile::Lockfile> {
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

    let config_str = fs::read_to_string(config_path)
        .await
        .map_err(|err| anyhow::anyhow!("Failed to read config at {}. Cause: {err}", config_path.display()))?;

    let config: gateway_config::Config = toml::from_str(&config_str)
        .map_err(|err| anyhow::anyhow!("Failed to parse config at {}. Cause: {err}", config_path.display()))?;

    let mut new_version_requirements: Vec<(String, semver::VersionReq)> = Vec::new();

    let mut extensions_from_config = config.extensions.unwrap_or_default();

    if extensions_from_config.is_empty() {
        report::no_extension_defined_in_config();
        std::process::exit(0)
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

    Ok(new_lockfile)
}
