#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Id {
    /// From where was this extension manifest retrieved? For example:
    /// - URL: https://grafbase.com/extensions
    /// - Local directory: file:///home/x/my-extension/build
    pub origin: String,
    pub name: String,
    pub version: semver::Version,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_is_compatible_with() {
        let expected = Id {
            origin: "https://grafbase.com/extensions".to_string(),
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
            origin: "file:///home/x/my-extension/build".to_string(),
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
