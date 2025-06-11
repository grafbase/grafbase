use std::path::Path;

use extension::*;
use url::Url;

/// The name of the directory where extensions are downloaded by `grafbase extension install` and loaded by the gateway.
pub const EXTENSION_DIR_NAME: &str = "grafbase_extensions";

pub async fn load_manifest(current_dir: Option<&Path>, mut url: Url) -> Result<Manifest, String> {
    if url
        .path_segments()
        .and_then(|mut segments| segments.next_back())
        .is_none_or(|last| last != "manifest.json")
    {
        url.path_segments_mut().unwrap().push("manifest.json");
    }

    let manifest: VersionedManifest = if url.scheme() == "file" {
        let mut path = url.to_file_path().map_err(|_| "Could not convert to file path")?;
        if path.is_relative() {
            let Some(current_dir) = current_dir else {
                return Err("Manifest file is relative and no current directory was provided".to_string());
            };
            path = current_dir.join(path);
        }
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
            r#type: Type::FieldResolver(FieldResolverType {
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
            event_filter: Default::default(),
        };
        tokio::fs::write(
            &manifest_path,
            serde_json::to_string(&expected.clone().into_versioned()).unwrap(),
        )
        .await
        .unwrap();
        let manifest = load_manifest(None, Url::from_file_path(dir.path()).unwrap())
            .await
            .unwrap();
        assert_eq!(manifest, expected);

        let other_manifest = load_manifest(None, Url::from_file_path(manifest_path).unwrap())
            .await
            .unwrap();
        assert_eq!(manifest, other_manifest);
    }
}
