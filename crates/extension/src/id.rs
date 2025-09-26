use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct Id {
    pub name: String,
    pub version: semver::Version,
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.name, self.version)
    }
}

impl FromStr for Id {
    type Err = semver::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, version) = s.rsplit_once('-').unwrap();
        Ok(Self {
            name: name.to_string(),
            version: semver::Version::parse(version)?,
        })
    }
}

impl Id {
    /// After loading extensions as defined in the Gateway configuration, we need to identify which
    /// one of those matches which directives in the federated GraphQL schema. So here `Self` is
    /// the extension loaded by the Gateway and `expected` the one defined in the SDL.
    pub fn is_compatible_with(&self, name: &str, version: &semver::VersionReq) -> bool {
        self.name == name && version.matches(&self.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_is_compatible_with() {
        let expected_name = "my-extension";
        let expected_version_req = semver::VersionReq::parse("^1.0.0").unwrap();

        let id = Id {
            name: "my-extension".to_string(),
            version: semver::Version::parse("1.0.0").unwrap(),
        };
        assert!(id.is_compatible_with(expected_name, &expected_version_req));

        let id = Id {
            name: "my-extension".to_string(),
            version: semver::Version::parse("1.1.0").unwrap(),
        };
        assert!(id.is_compatible_with(expected_name, &expected_version_req));

        let id = Id {
            name: "my-extension".to_string(),
            version: semver::Version::parse("1.0.1").unwrap(),
        };
        assert!(id.is_compatible_with(expected_name, &expected_version_req));

        let id = Id {
            name: "my-extension".to_string(),
            version: semver::Version::parse("2.0.0").unwrap(),
        };
        assert!(!id.is_compatible_with(expected_name, &expected_version_req));

        let id = Id {
            name: "another-extension".to_string(),
            version: semver::Version::parse("1.0.0").unwrap(),
        };
        assert!(!id.is_compatible_with(expected_name, &expected_version_req));
    }
}
