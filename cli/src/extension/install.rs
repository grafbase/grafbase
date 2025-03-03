use std::path::Path;

use futures::stream::FuturesUnordered;
use tokio::fs;
use tokio_stream::StreamExt as _;

use crate::output::report;

#[tokio::main]
pub(super) async fn execute() -> anyhow::Result<()> {
    let lockfile_path = Path::new(extension::lockfile::EXTENSION_LOCKFILE_NAME);
    let lockfile_str = fs::read_to_string(lockfile_path)
        .await
        .map_err(|err| anyhow::anyhow!("Failed to read lockfile at {}. Cause: {err}", lockfile_path.display()))?;

    let extension::lockfile::VersionedLockfile::V1(lockfile) = toml::from_str(&lockfile_str)
        .map_err(|err| anyhow::anyhow!("Failed to parse lockfile at {}. Cause: {err}", lockfile_path.display()))?;

    let extensions_directory = Path::new(extension_catalog::EXTENSION_DIR_NAME);
    let http_client = reqwest::Client::new();

    fs::create_dir_all(extensions_directory).await.map_err(|err| {
        anyhow::anyhow!(
            "Failed to create extensions directory at {}. Cause: {err}",
            extensions_directory.display()
        )
    })?;

    let mut futures = FuturesUnordered::new();

    report::extension_install_start(extensions_directory);

    let progress_bar = indicatif::ProgressBar::new(lockfile.extensions.len() as u64);

    for extension in lockfile.extensions {
        futures.push(extension_catalog::download_extension_from_registry(
            &http_client,
            extensions_directory,
            extension.name,
            extension.version,
        ));
    }

    while let Some(()) = futures.try_next().await? {
        progress_bar.inc(1);
    }

    progress_bar.finish_with_message("All extensions downloaded in ....");

    Ok(())
}
