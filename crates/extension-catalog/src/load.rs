use extension::*;
use url::Url;

pub async fn load_manifest(mut url: Url) -> Result<(Id, Manifest), String> {
    if url
        .path_segments()
        .and_then(|segments| segments.last())
        .is_none_or(|last| last != "manifest.json")
    {
        url.path_segments_mut().unwrap().push("manifest.json");
    }

    let manifest = if url.scheme() == "file" {
        let content = std::fs::read(url.path()).map_err(|err| err.to_string())?;
        serde_json::from_slice(&content).map_err(|err| err.to_string())?
    } else {
        reqwest::get(url.clone())
            .await
            .map_err(|err| err.to_string())?
            .json()
            .await
            .map_err(|err| err.to_string())?
    };

    let id = Id::from_url(url, &manifest);
    Ok((id, manifest))
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
            name: "my-extension".to_string(),
            version: "1.0.0".parse().unwrap(),
            kind: Kind::FieldResolver(FieldResolver {
                resolver_directives: vec!["resolver".to_string()],
            }),
            sdk_version: "0.3.0".parse().unwrap(),
            minimum_gateway_version: "0.90.0".parse().unwrap(),
            sdl: Some("directive foo on SCHEMA".to_string()),
        };
        tokio::fs::write(&manifest_path, serde_json::to_string(&expected).unwrap())
            .await
            .unwrap();
        let (id, manifest) = load_manifest(Url::from_file_path(dir.path()).unwrap()).await.unwrap();
        assert_eq!(id.origin, url::Url::from_file_path(dir.path()).unwrap());
        assert_eq!(id.name, manifest.name);
        assert_eq!(id.version, manifest.version);
        assert_eq!(manifest, expected);

        let (other_id, other_manifest) = load_manifest(Url::from_file_path(manifest_path).unwrap())
            .await
            .unwrap();
        assert_eq!((id, manifest), (other_id, other_manifest));
    }
}
