use std::path::Path;

use anyhow::Context as _;
use futures::stream::FuturesUnordered;
use tokio::{fs, io::AsyncWriteExt as _};
use tokio_stream::StreamExt as _;
use url::Url;

use crate::{extension::EXTENSION_WASM_MODULE_FILE_NAME, output::report};

#[tokio::main]
pub(super) async fn execute() -> anyhow::Result<()> {
    let lockfile_path = Path::new(extension::lockfile::EXTENSION_LOCKFILE_NAME);
    let lockfile_str = fs::read_to_string(lockfile_path)
        .await
        .map_err(|err| anyhow::anyhow!("Failed to read lockfile at {}. Cause: {err}", lockfile_path.display()))?;

    let extension::lockfile::VersionedLockfile::V1(lockfile) = toml::from_str(&lockfile_str)
        .map_err(|err| anyhow::anyhow!("Failed to parse lockfile at {}. Cause: {err}", lockfile_path.display()))?;

    let extensions_directory = Path::new(extension::directory::EXTENSION_DIR_NAME);
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
        futures.push(download_extension(
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

async fn download_extension(
    http_client: &reqwest::Client,
    extensions_dir: &Path,
    name: String,
    version: semver::Version,
) -> anyhow::Result<()> {
    const PUBLIC_EXTENSION_REGISTRY_URL: &str = "https://extensions.grafbase.com";

    let url: Url = PUBLIC_EXTENSION_REGISTRY_URL.parse().unwrap();

    let mut manifest_json_url = url.clone();
    manifest_json_url.set_path(&format!("/extensions/{name}/{version}/manifest.json"));

    let mut extension_wasm_url = url.clone();
    extension_wasm_url.set_path(&format!(
        "/extensions/{name}/{version}/{}",
        EXTENSION_WASM_MODULE_FILE_NAME
    ));

    let url_to_path = async |url: Url| -> anyhow::Result<()> {
        let last_url_segment = url.path_segments().unwrap().last().unwrap();
        let dir_path = extensions_dir.join(&name).join(version.to_string());
        let file_path = dir_path.join(last_url_segment);

        let response = http_client.get(url).send().await?;

        fs::create_dir_all(dir_path)
            .await
            .context("Failed to create subdirectory for extension")?;

        // Create the output file
        let mut file = fs::File::create(file_path).await?;

        // Stream the body bytes to the file
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.try_next().await? {
            file.write_all(&chunk).await?;
        }

        Ok(())
    };

    tokio::try_join!(url_to_path(manifest_json_url), url_to_path(extension_wasm_url)).map(|_| ())
}
