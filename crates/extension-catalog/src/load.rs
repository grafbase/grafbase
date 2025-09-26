use extension::*;
use url::Url;

/// The name of the directory where extensions are downloaded by `grafbase extension install` and loaded by the gateway.
pub const EXTENSION_DIR_NAME: &str = "grafbase_extensions";

pub async fn load_manifest(mut url: Url) -> Result<Manifest, String> {
    if url
        .path_segments()
        .and_then(|mut segments| segments.next_back())
        .is_none_or(|last| last != "manifest.json")
    {
        url.path_segments_mut().unwrap().push("manifest.json");
    }

    let manifest: VersionedManifest = if let Ok(path) = url.to_file_path() {
        let content = std::fs::read(&path).map_err(|err| err.to_string())?;
        serde_json::from_slice(&content).map_err(|err| err.to_string())?
    } else {
        reqwest::get(url)
            .await
            .map_err(|err| err.to_string())?
            .json()
            .await
            .map_err(|err| err.to_string())?
    };

    Ok(manifest.into_latest())
}
