mod event_filter;
mod permissions;

pub use event_filter::*;
pub use permissions::ExtensionPermission;

use crate::Id;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    pub id: Id,
    #[serde(alias = "kind")]
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
    #[serde(default)]
    pub permissions: Vec<ExtensionPermission>,
    // LEGACY Hooks event filter
    #[serde(skip_serializing_if = "Option::is_none", rename = "event_filter")]
    pub legacy_event_filter: Option<EventFilter>,
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
        matches!(self.r#type, Type::FieldResolver(_))
    }

    pub fn is_authentication(&self) -> bool {
        matches!(self.r#type, Type::Authentication(_))
    }

    pub fn is_hooks(&self) -> bool {
        matches!(self.r#type, Type::Hooks(_))
    }

    pub fn network_enabled(&self) -> bool {
        self.permissions.contains(&ExtensionPermission::Network)
    }

    pub fn stdout_enabled(&self) -> bool {
        self.permissions.contains(&ExtensionPermission::Stdout)
    }

    pub fn stderr_enabled(&self) -> bool {
        self.permissions.contains(&ExtensionPermission::Stderr)
    }

    pub fn environment_variables_enabled(&self) -> bool {
        self.permissions.contains(&ExtensionPermission::EnvironmentVariables)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, strum::EnumDiscriminants)]
pub enum Type {
    FieldResolver(FieldResolverType),
    SelectionSetResolver(Empty),
    Resolver(ResolverType),
    #[serde(alias = "Authenticator")]
    Authentication(Empty),
    Authorization(AuthorizationType),
    Hooks(HooksType),
    Contracts(Empty),
}

impl Type {
    pub fn is_resolver(&self) -> bool {
        matches!(self, Type::Resolver(_))
    }

    pub fn is_hooks(&self) -> bool {
        matches!(self, Type::Hooks(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FieldResolverType {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolver_directives: Option<Vec<String>>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AuthorizationType {
    #[serde(skip_serializing_if = "Option::is_none", alias = "authorization_directives")]
    pub directives: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_by: Option<Vec<AuthorizationGroupBy>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationGroupBy {
    Subgraph,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ResolverType {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directives: Option<Vec<String>>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HooksType {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_filter: Option<EventFilter>,
}

// Allows us to add fields later, as adding a value to an enum that doesn't have one would be
// breaking change if not handled carefully in serde.
#[derive(Default, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Empty {}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
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
            r#type: Type::FieldResolver(FieldResolverType {
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
            permissions: vec![
                ExtensionPermission::Stdout,
                ExtensionPermission::Stderr,
                ExtensionPermission::EnvironmentVariables,
                ExtensionPermission::Network,
            ],
            legacy_event_filter: None,
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
            r#type: Type::FieldResolver(FieldResolverType {
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
            permissions: Default::default(),
            legacy_event_filter: None,
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
            r#type: Type::FieldResolver(FieldResolverType {
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
            permissions: Default::default(),
            legacy_event_filter: None,
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
            permissions: Default::default(),
            legacy_event_filter: None,
        };

        assert_eq!(manifest, expected,)
    }

    #[test]
    fn authentication_empty_compatibility() {
        // Test authenticator with empty object (previous versions might have had different structures)
        let json = json!({
            "id": {"name": "auth", "version": "2.0.0"},
            "kind": {
                "Authentication": {}
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
            permissions: Default::default(),
            legacy_event_filter: None,
        };

        assert_eq!(manifest, expected,)
    }

    #[test]
    fn authorization_compatibility() {
        let manifest: Manifest = serde_json::from_value(json!({
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
        }))
        .unwrap();

        let expected = Manifest {
            id: Id {
                name: "authz".to_string(),
                version: semver::Version::new(1, 0, 0),
            },
            r#type: Type::Authorization(AuthorizationType {
                directives: Some(vec!["authorized".to_string(), "authenticated".to_string()]),
                group_by: None,
            }),
            sdk_version: semver::Version::new(0, 1, 0),
            minimum_gateway_version: semver::Version::new(0, 1, 0),
            sdl: None,
            description: "An authorization extension test".to_owned(),
            readme: None,
            homepage_url: Some("http://example.com/my-extension".parse().unwrap()),
            repository_url: None,
            license: None,
            permissions: Default::default(),
            legacy_event_filter: None,
        };
        assert_eq!(manifest, expected);

        let manifest2: Manifest = serde_json::from_value(json!({
            "id": {"name": "authz", "version": "1.0.0"},
            "kind": {
                "Authorization": {
                    "directives": ["authorized", "authenticated"]
                }
            },
            "sdk_version": "0.1.0",
            "minimum_gateway_version": "0.1.0",
            "description": "An authorization extension test",
            "homepage_url": "http://example.com/my-extension"
        }))
        .unwrap();
        assert_eq!(manifest, manifest2);
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
                directives: None,
                group_by: None,
            }),
            sdk_version: semver::Version::new(0, 1, 0),
            minimum_gateway_version: semver::Version::new(0, 1, 0),
            sdl: None,
            description: "An authorization extension test".to_owned(),
            readme: None,
            homepage_url: Some("http://example.com/my-extension".parse().unwrap()),
            repository_url: None,
            license: None,
            permissions: Default::default(),
            legacy_event_filter: None,
        };

        assert_eq!(manifest, expected);
    }

    #[test]
    fn authorization_compatibility_with_grouping() {
        let json = json!({
            "id": {"name": "authz", "version": "1.0.0"},
            "kind": {
                "Authorization": {
                    "directives": ["authorized", "authenticated"],
                    "group_by": ["subgraph"]
                }
            },
            "sdk_version": "0.1.0",
            "minimum_gateway_version": "0.1.0",
            "description": "An authorization extension test with grouping",
            "homepage_url": "http://example.com/my-extension"
        });

        let manifest: Manifest = serde_json::from_value(json).unwrap();

        let expected = Manifest {
            id: Id {
                name: "authz".to_string(),
                version: semver::Version::new(1, 0, 0),
            },
            r#type: Type::Authorization(AuthorizationType {
                directives: Some(vec!["authorized".to_string(), "authenticated".to_string()]),
                group_by: Some(vec![AuthorizationGroupBy::Subgraph]),
            }),
            sdk_version: semver::Version::new(0, 1, 0),
            minimum_gateway_version: semver::Version::new(0, 1, 0),
            sdl: None,
            description: "An authorization extension test with grouping".to_owned(),
            readme: None,
            homepage_url: Some("http://example.com/my-extension".parse().unwrap()),
            repository_url: None,
            license: None,
            permissions: Default::default(),
            legacy_event_filter: None,
        };

        assert_eq!(manifest, expected);
    }

    #[test]
    fn selection_set_resolver_compatbility() {
        let json = json!({
            "id": {"name": "selection-set", "version": "1.0.0"},
            "kind": {
                "SelectionSetResolver": {}
            },
            "sdk_version": "0.1.0",
            "minimum_gateway_version": "0.1.0",
            "description": "A selection-set resolver test",
            "homepage_url": "http://example.com/my-extension"
        });
        let manifest: Manifest = serde_json::from_value(json).unwrap();
        let expected = Manifest {
            id: Id {
                name: "selection-set".to_string(),
                version: semver::Version::new(1, 0, 0),
            },
            r#type: Type::SelectionSetResolver(Default::default()),
            sdk_version: semver::Version::new(0, 1, 0),
            minimum_gateway_version: semver::Version::new(0, 1, 0),
            sdl: None,
            description: "A selection-set resolver test".to_owned(),
            readme: None,
            homepage_url: Some("http://example.com/my-extension".parse().unwrap()),
            repository_url: None,
            license: None,
            permissions: Default::default(),
            legacy_event_filter: None,
        };
        assert_eq!(manifest, expected);
    }

    #[test]
    fn resolver_compatbility() {
        let json = json!({
            "id": {"name": "selection-set", "version": "1.0.0"},
            "kind": {
                "Resolver": {}
            },
            "sdk_version": "0.1.0",
            "minimum_gateway_version": "0.1.0",
            "description": "A selection-set resolver test",
            "homepage_url": "http://example.com/my-extension"
        });
        let manifest: Manifest = serde_json::from_value(json).unwrap();
        let expected = Manifest {
            id: Id {
                name: "selection-set".to_string(),
                version: semver::Version::new(1, 0, 0),
            },
            r#type: Type::Resolver(Default::default()),
            sdk_version: semver::Version::new(0, 1, 0),
            minimum_gateway_version: semver::Version::new(0, 1, 0),
            sdl: None,
            description: "A selection-set resolver test".to_owned(),
            readme: None,
            homepage_url: Some("http://example.com/my-extension".parse().unwrap()),
            repository_url: None,
            license: None,
            permissions: Default::default(),
            legacy_event_filter: None,
        };
        assert_eq!(manifest, expected);

        let json = json!({
            "id": {"name": "selection-set", "version": "1.0.0"},
            "kind": {
                "Resolver": {
                    "directives": ["rest"]
                }
            },
            "sdk_version": "0.1.0",
            "minimum_gateway_version": "0.1.0",
            "description": "A selection-set resolver test",
            "homepage_url": "http://example.com/my-extension"
        });
        let manifest: Manifest = serde_json::from_value(json).unwrap();
        let expected = Manifest {
            id: Id {
                name: "selection-set".to_string(),
                version: semver::Version::new(1, 0, 0),
            },
            r#type: Type::Resolver(ResolverType {
                directives: Some(vec!["rest".to_string()]),
            }),
            sdk_version: semver::Version::new(0, 1, 0),
            minimum_gateway_version: semver::Version::new(0, 1, 0),
            sdl: None,
            description: "A selection-set resolver test".to_owned(),
            readme: None,
            homepage_url: Some("http://example.com/my-extension".parse().unwrap()),
            repository_url: None,
            license: None,
            permissions: Default::default(),
            legacy_event_filter: None,
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

    #[test]
    fn hooks_compatibility_with_legacy_root_event_filter() {
        // Test legacy format where event_filter was at the root level
        let json = json!({
            "id": {"name": "hooks-test", "version": "1.0.0"},
            "kind": {
                "Hooks": {}
            },
            "sdk_version": "0.1.0",
            "minimum_gateway_version": "0.1.0",
            "description": "A hooks extension test",
            "homepage_url": "http://example.com/my-extension",
            "event_filter": {
                "All": null
            }
        });

        let manifest: Manifest = serde_json::from_value(json).unwrap();

        let expected = Manifest {
            id: Id {
                name: "hooks-test".to_string(),
                version: semver::Version::new(1, 0, 0),
            },
            r#type: Type::Hooks(HooksType { event_filter: None }),
            sdk_version: semver::Version::new(0, 1, 0),
            minimum_gateway_version: semver::Version::new(0, 1, 0),
            sdl: None,
            description: "A hooks extension test".to_owned(),
            readme: None,
            homepage_url: Some("http://example.com/my-extension".parse().unwrap()),
            repository_url: None,
            license: None,
            permissions: Default::default(),
            legacy_event_filter: Some(EventFilter::All),
        };

        assert_eq!(manifest, expected);

        // Test with specific event types
        let json = json!({
            "id": {"name": "hooks-test", "version": "1.0.0"},
            "kind": {
                "Hooks": {}
            },
            "sdk_version": "0.1.0",
            "minimum_gateway_version": "0.1.0",
            "description": "A hooks extension test",
            "homepage_url": "http://example.com/my-extension",
            "event_filter": {
                "Types": ["operation", "subgraph_request", "http_request"]
            }
        });

        let manifest: Manifest = serde_json::from_value(json).unwrap();

        let expected = Manifest {
            id: Id {
                name: "hooks-test".to_string(),
                version: semver::Version::new(1, 0, 0),
            },
            r#type: Type::Hooks(HooksType { event_filter: None }),
            sdk_version: semver::Version::new(0, 1, 0),
            minimum_gateway_version: semver::Version::new(0, 1, 0),
            sdl: None,
            description: "A hooks extension test".to_owned(),
            readme: None,
            homepage_url: Some("http://example.com/my-extension".parse().unwrap()),
            repository_url: None,
            license: None,
            permissions: Default::default(),
            legacy_event_filter: Some(EventFilter::Types(vec![
                EventType::Operation,
                EventType::SubgraphRequest,
                EventType::HttpRequest,
            ])),
        };

        assert_eq!(manifest, expected);
    }

    #[test]
    fn hooks_compatibility_with_nested_event_filter() {
        // Test new format where event_filter is nested inside the Hooks type
        let json = json!({
            "id": {"name": "hooks-test", "version": "2.0.0"},
            "kind": {
                "Hooks": {
                    "event_filter": {
                        "All": null
                    }
                }
            },
            "sdk_version": "0.1.0",
            "minimum_gateway_version": "0.1.0",
            "description": "A hooks extension test with nested filter",
            "homepage_url": "http://example.com/my-extension"
        });

        let manifest: Manifest = serde_json::from_value(json).unwrap();

        let expected = Manifest {
            id: Id {
                name: "hooks-test".to_string(),
                version: semver::Version::new(2, 0, 0),
            },
            r#type: Type::Hooks(HooksType {
                event_filter: Some(EventFilter::All),
            }),
            sdk_version: semver::Version::new(0, 1, 0),
            minimum_gateway_version: semver::Version::new(0, 1, 0),
            sdl: None,
            description: "A hooks extension test with nested filter".to_owned(),
            readme: None,
            homepage_url: Some("http://example.com/my-extension".parse().unwrap()),
            repository_url: None,
            license: None,
            permissions: Default::default(),
            legacy_event_filter: None,
        };

        assert_eq!(manifest, expected);

        // Test with specific event types in nested format
        let json = json!({
            "id": {"name": "hooks-test", "version": "2.0.0"},
            "kind": {
                "Hooks": {
                    "event_filter": {
                        "Types": ["operation", "extension"]
                    }
                }
            },
            "sdk_version": "0.1.0",
            "minimum_gateway_version": "0.1.0",
            "description": "A hooks extension test with nested filter",
            "homepage_url": "http://example.com/my-extension"
        });

        let manifest: Manifest = serde_json::from_value(json).unwrap();

        let expected = Manifest {
            id: Id {
                name: "hooks-test".to_string(),
                version: semver::Version::new(2, 0, 0),
            },
            r#type: Type::Hooks(HooksType {
                event_filter: Some(EventFilter::Types(vec![EventType::Operation, EventType::Extension])),
            }),
            sdk_version: semver::Version::new(0, 1, 0),
            minimum_gateway_version: semver::Version::new(0, 1, 0),
            sdl: None,
            description: "A hooks extension test with nested filter".to_owned(),
            readme: None,
            homepage_url: Some("http://example.com/my-extension".parse().unwrap()),
            repository_url: None,
            license: None,
            permissions: Default::default(),
            legacy_event_filter: None,
        };

        assert_eq!(manifest, expected);
    }
}
