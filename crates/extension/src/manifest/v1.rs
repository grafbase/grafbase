mod permissions;

pub use enumflags2::BitFlags;
pub use permissions::ExtensionPermission;

use crate::Id;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    pub id: Id,
    #[serde(rename = "kind")]
    pub r#type: Type,
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
    #[serde(default, with = "permissions::serializing")]
    pub permissions: BitFlags<ExtensionPermission>,
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
        matches!(self.r#type, Type::Resolver(_))
    }

    pub fn is_authenticator(&self) -> bool {
        matches!(self.r#type, Type::Authentication(_))
    }

    pub fn network_enabled(&self) -> bool {
        self.permissions.contains(ExtensionPermission::Network)
    }

    pub fn stdout_enabled(&self) -> bool {
        self.permissions.contains(ExtensionPermission::Stdout)
    }

    pub fn stderr_enabled(&self) -> bool {
        self.permissions.contains(ExtensionPermission::Stderr)
    }

    pub fn environment_variables_enabled(&self) -> bool {
        self.permissions.contains(ExtensionPermission::EnvironmentVariables)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, strum::EnumDiscriminants)]
pub enum Type {
    #[serde(rename = "FieldResolver")]
    Resolver(ResolverType),
    #[serde(rename = "Authenticator")]
    Authentication(Empty),
    Authorization(AuthorizationType),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ResolverType {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolver_directives: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AuthorizationType {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_directives: Option<Vec<String>>,
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
    fn permissions() {
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
            "permissions": ["stdout", "stderr", "environment_variables", "network"]
        });

        let manifest: Manifest = serde_json::from_value(json).unwrap();

        let expected = Manifest {
            id: Id {
                name: "test".to_string(),
                version: semver::Version::new(1, 0, 0),
            },
            r#type: Type::Resolver(ResolverType {
                resolver_directives: Some(vec!["custom".to_string()]),
            }),
            sdk_version: semver::Version::new(0, 1, 0),
            minimum_gateway_version: semver::Version::new(0, 1, 0),
            sdl: Some("directive @custom on FIELD_DEFINITION".to_string()),
            description: "Mandatory description".to_owned(),
            readme: None,
            homepage_url: Some("http://example.com/my-extension".parse().unwrap()),
            repository_url: None,
            license: None,
            permissions: BitFlags::all(),
        };

        assert_eq!(manifest, expected,);
    }

    #[test]
    fn field_resolver() {
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

        let expected = Manifest {
            id: Id {
                name: "test".to_string(),
                version: semver::Version::new(1, 0, 0),
            },
            r#type: Type::Resolver(ResolverType {
                resolver_directives: Some(vec!["custom".to_string()]),
            }),
            sdk_version: semver::Version::new(0, 1, 0),
            minimum_gateway_version: semver::Version::new(0, 1, 0),
            sdl: Some("directive @custom on FIELD_DEFINITION".to_string()),
            description: "Mandatory description".to_owned(),
            readme: None,
            homepage_url: Some("http://example.com/my-extension".parse().unwrap()),
            repository_url: None,
            license: None,
            permissions: BitFlags::empty(),
        };

        assert_eq!(manifest, expected,);
    }

    #[test]
    fn field_resolver_without_directives() {
        // Test compatibility with previous snake_case/camelCase variations
        let json = json!({
            "id": {"name": "test", "version": "1.0.0"},
            "kind": {
                "FieldResolver": {}
            },
            "sdl": "directive @custom on FIELD_DEFINITION",
            "sdk_version": "0.1.0",
            "minimum_gateway_version": "0.1.0",
            "description": "Mandatory description",
            "homepage_url": "http://example.com/my-extension",
        });

        let manifest: Manifest = serde_json::from_value(json).unwrap();

        let expected = Manifest {
            id: Id {
                name: "test".to_string(),
                version: semver::Version::new(1, 0, 0),
            },
            r#type: Type::Resolver(ResolverType {
                resolver_directives: None,
            }),
            sdk_version: semver::Version::new(0, 1, 0),
            minimum_gateway_version: semver::Version::new(0, 1, 0),
            sdl: Some("directive @custom on FIELD_DEFINITION".to_string()),
            description: "Mandatory description".to_owned(),
            readme: None,
            homepage_url: Some("http://example.com/my-extension".parse().unwrap()),
            repository_url: None,
            license: None,
            permissions: BitFlags::empty(),
        };

        assert_eq!(manifest, expected,);
    }

    #[test]
    fn authenticator_empty_compatibility() {
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

        let expected = Manifest {
            id: Id {
                name: "auth".to_string(),
                version: semver::Version::new(2, 0, 0),
            },
            r#type: Type::Authentication(Empty {}),
            sdk_version: semver::Version::new(0, 1, 0),
            minimum_gateway_version: semver::Version::new(0, 1, 0),
            sdl: None,
            description: "An extension in a test".to_owned(),
            readme: None,
            homepage_url: Some("http://example.com/my-extension".parse().unwrap()),
            repository_url: None,
            license: None,
            permissions: BitFlags::empty(),
        };

        assert_eq!(manifest, expected,)
    }

    #[test]
    fn authorization_compatibility() {
        let json = json!({
            "id": {"name": "authz", "version": "1.0.0"},
            "kind": {
                "Authorization": {
                    "authorization_directives": ["authorized", "authenticated"]
                }
            },
            "sdk_version": "0.1.0",
            "minimum_gateway_version": "0.1.0",
            "description": "An authorization extension test",
            "homepage_url": "http://example.com/my-extension"
        });

        let manifest: Manifest = serde_json::from_value(json).unwrap();

        let expected = Manifest {
            id: Id {
                name: "authz".to_string(),
                version: semver::Version::new(1, 0, 0),
            },
            r#type: Type::Authorization(AuthorizationType {
                authorization_directives: Some(vec!["authorized".to_string(), "authenticated".to_string()]),
            }),
            sdk_version: semver::Version::new(0, 1, 0),
            minimum_gateway_version: semver::Version::new(0, 1, 0),
            sdl: None,
            description: "An authorization extension test".to_owned(),
            readme: None,
            homepage_url: Some("http://example.com/my-extension".parse().unwrap()),
            repository_url: None,
            license: None,
            permissions: BitFlags::empty(),
        };

        assert_eq!(manifest, expected);
    }

    #[test]
    fn authorization_compatibility_without_directives() {
        let json = json!({
            "id": {"name": "authz", "version": "1.0.0"},
            "kind": {
                "Authorization": {}
            },
            "sdk_version": "0.1.0",
            "minimum_gateway_version": "0.1.0",
            "description": "An authorization extension test",
            "homepage_url": "http://example.com/my-extension"
        });

        let manifest: Manifest = serde_json::from_value(json).unwrap();

        let expected = Manifest {
            id: Id {
                name: "authz".to_string(),
                version: semver::Version::new(1, 0, 0),
            },
            r#type: Type::Authorization(AuthorizationType {
                authorization_directives: None,
            }),
            sdk_version: semver::Version::new(0, 1, 0),
            minimum_gateway_version: semver::Version::new(0, 1, 0),
            sdl: None,
            description: "An authorization extension test".to_owned(),
            readme: None,
            homepage_url: Some("http://example.com/my-extension".parse().unwrap()),
            repository_url: None,
            license: None,
            permissions: BitFlags::empty(),
        };

        assert_eq!(manifest, expected);
    }

    #[test]
    fn missing_optional_fields() {
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
