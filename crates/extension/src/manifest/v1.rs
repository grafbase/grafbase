use crate::Id;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    pub id: Id,
    pub kind: Kind,
    pub sdk_version: semver::Version,
    pub minimum_gateway_version: semver::Version,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sdl: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage_url: Option<url::Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_url: Option<url::Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}

impl Manifest {
    pub fn name(&self) -> &str {
        &self.id.name
    }

    pub fn version(&self) -> &semver::Version {
        &self.id.version
    }

    pub fn into_versioned(self) -> super::VersionedManifest {
        super::VersionedManifest::V1(self)
    }

    pub fn is_resolver(&self) -> bool {
        matches!(self.kind, Kind::FieldResolver(_))
    }

    pub fn is_authenticator(&self) -> bool {
        matches!(self.kind, Kind::Authenticator(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, strum::EnumDiscriminants)]
pub enum Kind {
    FieldResolver(FieldResolver),
    Authenticator(Empty),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FieldResolver {
    pub resolver_directives: Vec<String>,
}

// Allows us to add fields later, as adding a value to an enum that doesn't have one would be
// breaking change if not handled carefully in serde.
#[derive(Default, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Empty {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn v1_field_resolver_with_serde_rename() {
        // Test compatibility with previous snake_case/camelCase variations
        let json = json!({
            "id": {"name": "test", "version": "1.0.0"},
            "kind": {
                "FieldResolver": {
                    "resolver_directives": ["custom"]
                }
            },
            "sdl": "directive @custom on FIELD_DEFINITION",
            "sdk_version": "0.1.0",
            "minimum_gateway_version": "0.1.0",
            "description": "Mandatory description",
            "homepage_url": "http://example.com/my-extension",
        });

        let manifest: Manifest = serde_json::from_value(json).unwrap();
        assert_eq!(
            manifest,
            Manifest {
                id: Id {
                    name: "test".to_string(),
                    version: semver::Version::new(1, 0, 0)
                },
                kind: Kind::FieldResolver(FieldResolver {
                    resolver_directives: vec!["custom".to_string()]
                }),
                sdk_version: semver::Version::new(0, 1, 0),
                minimum_gateway_version: semver::Version::new(0, 1, 0),
                sdl: Some("directive @custom on FIELD_DEFINITION".to_string()),
                description: "Mandatory description".to_owned(),
                readme: None,
                homepage_url: Some("http://example.com/my-extension".parse().unwrap()),
                repository_url: None,
                license: None
            }
        );
    }

    #[test]
    fn v1_authenticator_empty_compatibility() {
        // Test authenticator with empty object (previous versions might have had different structures)
        let json = json!({
            "id": {"name": "auth", "version": "2.0.0"},
            "kind": {
                "Authenticator": {}
            },
            "sdk_version": "0.1.0",
            "minimum_gateway_version": "0.1.0",
            "description": "An extension in a test",
            "homepage_url": "http://example.com/my-extension"
        });

        let manifest: Manifest = serde_json::from_value(json).unwrap();
        assert_eq!(
            manifest,
            Manifest {
                id: Id {
                    name: "auth".to_string(),
                    version: semver::Version::new(2, 0, 0)
                },
                kind: Kind::Authenticator(Empty {}),
                sdk_version: semver::Version::new(0, 1, 0),
                minimum_gateway_version: semver::Version::new(0, 1, 0),
                sdl: None,
                description: "An extension in a test".to_owned(),
                readme: None,
                homepage_url: Some("http://example.com/my-extension".parse().unwrap()),
                repository_url: None,
                license: None
            }
        )
    }

    #[test]
    fn v1_missing_optional_fields() {
        // Test older versions that might not have had the sdl field
        let json = json!({
            "id": {"name": "legacy", "version": "0.5.0"},
            "kind": {
                "FieldResolver": {
                    "resolver_directives": []
                }
            },
            "sdk_version": "0.0.9",
            "minimum_gateway_version": "0.0.9",
            "description": "mandatory description"
        });

        let manifest: Manifest = serde_json::from_value(json).unwrap();
        assert!(manifest.sdl.is_none());
    }
}
