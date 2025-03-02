use extension::*;
use futures_util::TryStreamExt as _;
use std::{borrow::Cow, io, path::Path};
use tokio::{fs, io::AsyncWriteExt as _};
use url::Url;

pub const EXTENSION_DIR_NAME: &str = "grafbase_extensions";
pub const PUBLIC_EXTENSION_REGISTRY_URL: &str = "https://extensions.grafbase.com";

pub async fn load_manifest(mut url: Url) -> Result<Manifest, String> {
    if url
        .path_segments()
        .and_then(|segments| segments.last())
        .is_none_or(|last| last != "manifest.json")
    {
        url.path_segments_mut().unwrap().push("manifest.json");
    }

    let manifest: VersionedManifest = if url.scheme() == "file" {
        let content = std::fs::read(url.to_file_path().map_err(|_| "Could not convert to file path")?)
            .map_err(|err| err.to_string())?;
        serde_json::from_slice(&content).map_err(|err| err.to_string())?
    } else {
        reqwest::get(url.clone())
            .await
            .map_err(|err| err.to_string())?
            .json()
            .await
            .map_err(|err| err.to_string())?
    };

    Ok(manifest.into_latest())
}

pub async fn download_extension_from_registry_if_needed(
    http_client: &reqwest::Client,
    extensions_dir: &Path,
    extension_name: String,
    version: semver::Version,
    registry_base_url: &Url,
) -> Result<(), Report> {
    download_extension_from_registry_impl(
        http_client,
        extensions_dir,
        extension_name,
        version,
        true,
        registry_base_url,
    )
    .await
}

pub async fn download_extension_from_registry(
    http_client: &reqwest::Client,
    extensions_dir: &Path,
    extension_name: String,
    version: semver::Version,
    registry_base_url: &Url,
) -> Result<(), Report> {
    download_extension_from_registry_impl(
        http_client,
        extensions_dir,
        extension_name,
        version,
        false,
        registry_base_url,
    )
    .await
}

async fn download_extension_from_registry_impl(
    http_client: &reqwest::Client,
    extensions_dir: &Path,
    extension_name: String,
    version: semver::Version,
    lazy: bool,
    registry_base_url: &Url,
) -> Result<(), Report> {
    let mut manifest_json_url = registry_base_url.clone();
    manifest_json_url.set_path(&format!("/extensions/{extension_name}/{version}/manifest.json"));

    let mut extension_wasm_url = registry_base_url.clone();
    extension_wasm_url.set_path(&format!("/extensions/{extension_name}/{version}/extension.wasm",));

    let url_to_path = async |url: Url| -> Result<(), Report> {
        let last_url_segment = url.path_segments().unwrap().last().unwrap();
        let dir_path = extensions_dir.join(&extension_name).join(version.to_string());
        let file_path = dir_path.join(last_url_segment);

        if lazy && file_path.exists() {
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
    };

    tokio::try_join!(url_to_path(manifest_json_url), url_to_path(extension_wasm_url)).map(|_| ())
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct Report(Cow<'static, str>);

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

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[tokio::test]
    async fn test_load_manifest() {
        let dir = tempdir().unwrap();

        let manifest_path = dir.path().join("manifest.json");
        let expected = Manifest {
            id: Id {
                name: "my-extension".to_string(),
                version: "1.0.0".parse().unwrap(),
            },
            kind: Kind::FieldResolver(FieldResolver {
                resolver_directives: Some(vec!["resolver".to_string()]),
            }),
            sdk_version: "0.3.0".parse().unwrap(),
            minimum_gateway_version: "0.90.0".parse().unwrap(),
            sdl: Some("directive foo on SCHEMA".to_string()),
            license: None,
            readme: None,
            description: "My extension".to_string(),
            homepage_url: None,
            repository_url: None,
            permissions: Default::default(),
        };
        tokio::fs::write(
            &manifest_path,
            serde_json::to_string(&expected.clone().into_versioned()).unwrap(),
        )
        .await
        .unwrap();
        let manifest = load_manifest(Url::from_file_path(dir.path()).unwrap()).await.unwrap();
        assert_eq!(manifest, expected);

        let other_manifest = load_manifest(Url::from_file_path(manifest_path).unwrap())
            .await
            .unwrap();
        assert_eq!(manifest, other_manifest);
    }
}
