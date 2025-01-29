use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct Id {
    /// From where was this extension manifest retrieved? For example:
    /// - URL: https://grafbase.com/extensions
    /// - Local directory: file:///home/x/my-extension/build
    ///
    /// It should not include the name/version
    pub origin: Url,
    pub name: String,
    pub version: semver::Version,
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.origin
                .clone()
                .join(&self.name)
                .unwrap()
                .join(&self.version.to_string())
                .unwrap()
        )
    }
}

impl Id {
    /// After loading extensions as defined in the Gateway configuration, we need to identify which
    /// one of those matches which directives in the federated GraphQL schema. So here `Self` is
    /// the extension loaded by the Gateway and `expected` the one defined in the SDL.
    pub fn is_compatible_with(&self, expected: &Id) -> bool {
        if self.origin != expected.origin || self.name != expected.name {
            return false;
        }
        let expected_version = semver::Comparator {
            op: semver::Op::Caret,
            major: expected.version.major,
            minor: Some(expected.version.minor),
            patch: Some(expected.version.patch),
            pre: Default::default(),
        };
        expected_version.matches(&self.version)
    }

    pub fn from_url(mut url: url::Url, name: String, version: semver::Version) -> Self {
        if url.path_segments().and_then(|seq| seq.last()) == Some("manifest.json") {
            url.path_segments_mut().unwrap().pop();
        }
        if url
            .path_segments()
            .and_then(|seg| seg.last())
            .and_then(|last| {
                let version = version.to_string();
                last.strip_suffix(&version).map(|last| last.is_empty() || last == "v")
            })
            .unwrap_or_default()
        {
            url.path_segments_mut().unwrap().pop();
        }
        if url.path_segments().and_then(|seg| seg.last()) == Some(&name) {
            url.path_segments_mut().unwrap().pop();
        }
        if let Ok(mut seg) = url.path_segments_mut() {
            seg.pop_if_empty();
        }
        Self {
            origin: url,
            name,
            version,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_url() {
        let manifest = crate::Manifest {
            name: "test-ext".to_string(),
            version: semver::Version::parse("1.2.3").unwrap(),
            kind: crate::Kind::FieldResolver(crate::FieldResolver {
                resolver_directives: Vec::new(),
            }),
            sdk_version: "0.3.0".parse().unwrap(),
            minimum_gateway_version: "0.90.0".parse().unwrap(),
            sdl: None,
        };

        // Test basic URL
        let name = "test-ext".to_string();
        let version: semver::Version = "1.2.3".parse().unwrap();
        let url = url::Url::parse("https://example.com/extensions").unwrap();
        let id = Id::from_url(url, name.clone(), version.clone());
        assert_eq!(id.origin, "https://example.com/extensions".parse().unwrap());
        assert_eq!(id.name, "test-ext");
        assert_eq!(id.version, manifest.version);

        // Test URL with manifest.json
        let url = url::Url::parse("https://example.com/extensions/manifest.json").unwrap();
        let id = Id::from_url(url, name.clone(), version.clone());
        assert_eq!(id.origin, "https://example.com/extensions".parse().unwrap());

        // Test URL with version
        let url = url::Url::parse("https://example.com/extensions/v1.2.3").unwrap();
        let id = Id::from_url(url, name.clone(), version.clone());
        assert_eq!(id.origin, "https://example.com/extensions".parse().unwrap());

        // Test URL with name and version
        let url = url::Url::parse("https://example.com/extensions/test-ext/1.2.3").unwrap();
        let id = Id::from_url(url, name.clone(), version.clone());
        assert_eq!(id.origin, "https://example.com/extensions".parse().unwrap());

        // Test URL with name and version and manifest.json
        let url = url::Url::parse("https://example.com/extensions/test-ext/1.2.3/manifest.json").unwrap();
        let id = Id::from_url(url, name.clone(), version.clone());
        assert_eq!(id.origin, "https://example.com/extensions".parse().unwrap());

        // Test URL with name and version and manifest.json 2
        let url = url::Url::parse("https://example.com/extensions/test-ext/v1.2.3/manifest.json").unwrap();
        let id = Id::from_url(url, name.clone(), version.clone());
        assert_eq!(id.origin, "https://example.com/extensions".parse().unwrap());
    }

    #[test]
    fn id_is_compatible_with() {
        let expected = Id {
            origin: "https://grafbase.com/extensions".parse().unwrap(),
            name: "my-extension".to_string(),
            version: semver::Version::parse("1.0.0").unwrap(),
        };
        let id = expected.clone();
        assert!(id.is_compatible_with(&expected));

        let id = Id {
            version: semver::Version::parse("1.1.0").unwrap(),
            ..expected.clone()
        };
        assert!(id.is_compatible_with(&expected));

        let id = Id {
            version: semver::Version::parse("1.0.1").unwrap(),
            ..expected.clone()
        };
        assert!(id.is_compatible_with(&expected));

        let id = Id {
            version: semver::Version::parse("2.0.0").unwrap(),
            ..expected.clone()
        };
        assert!(!id.is_compatible_with(&expected));

        let id = Id {
            origin: "file:///home/x/my-extension/build".parse().unwrap(),
            ..expected.clone()
        };
        assert!(!id.is_compatible_with(&expected));

        let id = Id {
            name: "another-extension".to_string(),
            ..expected.clone()
        };
        assert!(!id.is_compatible_with(&expected));
    }
}
