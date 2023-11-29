use std::borrow::Cow;

use async_graphql_parser::types as ast;
use async_graphql_value::ConstValue;

pub(super) fn ingest_schema_definitions(document: &ast::ServiceDocument) -> FederationDirectivesMatcher<'_> {
    document
        .definitions
        .iter()
        .filter_map(|def| {
            if let ast::TypeSystemDefinition::Schema(schema) = def {
                Some(schema)
            } else {
                None
            }
        })
        .flat_map(|schema_definition| schema_definition.node.directives.iter())
        .map(|d| &d.node)
        .find(|d| FederationDirectivesMatcher::is_federation_directive(d))
        .map(FederationDirectivesMatcher::new)
        .unwrap_or_default()
}

/// This struct is the source of truth for matching federation directives by name when ingesting a
/// subgraph's GraphQL SDL.
///
/// The names of federation directives are influenced by `@link` directives on schema definitions
/// or extensions in two ways:
///
/// - Imports in link directives bring the directives in scope, with optional renaming.
///   Example: `@link(url: "...", import: [{ name: "@shareable", as: "@federationShareable"}])
///   Example: `@link(url: "...", import: ["@key"])`
///
/// - The `as` argument: `@link(url: "...", as: "compositionDirectives")
///   - In the absence of an `@link` or `as` argument, all directives are in scope prefixed with
///   `@federation__`, for example `@federation__shareable`.
///   - With an `@link(as: "something")`, they are in scope under the `@something__` prefix.
///
/// Last rule: if a directive is `import`ed, it is no longer available under the prefix.
#[derive(Debug)]
pub(crate) struct FederationDirectivesMatcher<'a> {
    shareable: Cow<'a, str>,
    key: Cow<'a, str>,
    external: Cow<'a, str>,
    provides: Cow<'a, str>,
    requires: Cow<'a, str>,
    inaccessible: Cow<'a, str>,
    interface_object: Cow<'a, str>,
    r#override: Cow<'a, str>,
}

const DEFAULT_FEDERATION_PREFIX: &str = "federation__";

impl Default for FederationDirectivesMatcher<'_> {
    fn default() -> Self {
        FederationDirectivesMatcher {
            shareable: Cow::Borrowed("shareable"),
            key: Cow::Borrowed("key"),
            external: Cow::Borrowed("external"),
            provides: Cow::Borrowed("provides"),
            requires: Cow::Borrowed("requires"),
            inaccessible: Cow::Borrowed("inaccessible"),
            interface_object: Cow::Borrowed("interfaceObject"),
            r#override: Cow::Borrowed("override"),
        }
    }
}

impl<'a> FederationDirectivesMatcher<'a> {
    pub(crate) fn is_federation_directive(directive: &ast::ConstDirective) -> bool {
        if directive.name.node != "link" {
            return false;
        }

        directive
            .get_argument("url")
            .map(|url| match &url.node {
                ConstValue::String(s) => s.contains("dev/federation/v2"),
                _ => false,
            })
            .unwrap_or_default()
    }

    /// Matcher for federation directives in a given subgraph. See [FederationDirectivesMatcher] for more docs.        
    pub(crate) fn new(directive: &'a ast::ConstDirective) -> FederationDirectivesMatcher<'a> {
        let mut r#as = None;
        let mut imported: Vec<(&str, &str)> = Vec::new();

        for (arg_name, arg_value) in &directive.arguments {
            match (arg_name.node.as_str(), &arg_value.node) {
                ("as", ConstValue::String(value)) => r#as = Some(value.as_str()),
                ("import", ConstValue::List(imports)) => read_imports(imports, &mut imported),
                _ => (),
            }
        }

        let federation_prefix = r#as
            .map(|prefix| Cow::Owned(format!("{prefix}__")))
            .unwrap_or(Cow::Borrowed(DEFAULT_FEDERATION_PREFIX));
        let final_name = |directive_name: &str| {
            imported
                .iter()
                .find(|(original, _alias)| *original == directive_name)
                .map(|(_, alias)| Cow::Borrowed(*alias))
                .unwrap_or_else(|| Cow::Owned(format!("{federation_prefix}{directive_name}")))
        };

        FederationDirectivesMatcher {
            shareable: final_name("shareable"),
            key: final_name("key"),
            external: final_name("external"),
            provides: final_name("provides"),
            requires: final_name("requires"),
            inaccessible: final_name("inaccessible"),
            interface_object: final_name("interfaceObject"),
            r#override: final_name("override"),
        }
    }

    pub(crate) fn is_external(&self, directive_name: &str) -> bool {
        self.external == directive_name
    }

    pub(crate) fn is_interface_object(&self, directive_name: &str) -> bool {
        self.interface_object == directive_name
    }

    pub(crate) fn is_shareable(&self, directive_name: &str) -> bool {
        self.shareable == directive_name
    }

    pub(crate) fn is_override(&self, directive_name: &str) -> bool {
        self.r#override == directive_name
    }

    pub(crate) fn is_requires(&self, directive_name: &str) -> bool {
        self.requires == directive_name
    }

    pub(crate) fn is_provides(&self, directive_name: &str) -> bool {
        self.provides == directive_name
    }

    pub(crate) fn is_key(&self, directive_name: &str) -> bool {
        self.key == directive_name
    }

    pub(crate) fn is_inaccessible(&self, directive_name: &str) -> bool {
        self.inaccessible == directive_name
    }
}

fn read_imports<'a>(ast_imports: &'a [ConstValue], out: &mut Vec<(&'a str, &'a str)>) {
    for import in ast_imports {
        match import {
            ConstValue::String(import) => {
                let import = import.trim_start_matches('@');
                out.push((import, import));
            }
            ConstValue::Object(obj) => {
                if let Some(ConstValue::String(name)) = obj.get("name") {
                    let alias = obj.get("as").and_then(|value| match value {
                        ConstValue::String(s) => Some(s),
                        _ => None,
                    });
                    out.push((
                        name.trim_start_matches('@'),
                        alias.unwrap_or(name).trim_start_matches('@'),
                    ));
                }
            }
            _ => (),
        }
    }
}

#[cfg(test)]
mod federation_directives_matcher_tests {
    use datatest_stable as _;
    use miette as _;
    use similar as _;

    use super::*;

    fn with_matcher_for_schema(graphql_sdl: &str, test: impl FnOnce(FederationDirectivesMatcher<'_>)) {
        let ast = async_graphql_parser::parse_schema(graphql_sdl).unwrap();
        let matcher = ingest_schema_definitions(&ast);
        test(matcher);
    }

    #[test]
    fn no_link_declaration() {
        with_matcher_for_schema("type Irrelevant { id: ID! }", |matcher| {
            assert!(matcher.is_shareable("shareable"));
            assert!(matcher.is_key("key"));
            assert!(!matcher.is_key("@key"));
            assert!(!matcher.is_key("federation__key"));
            assert!(!matcher.is_shareable("federation__shareable"));
        });
    }

    #[test]
    fn bare_link_declaration() {
        let schema = r#"extend schema @link(url: "https://specs.apollo.dev/federation/v2.3")"#;
        with_matcher_for_schema(schema, |matcher| {
            assert!(matcher.is_key("federation__key"));
            assert!(matcher.is_shareable("federation__shareable"));
            assert!(!matcher.is_key("key"));
            assert!(!matcher.is_key("@key"));
            assert!(!matcher.is_shareable("shareable"));
        });
    }

    #[test]
    fn irrelevant_link_declaration() {
        let schema = r#"extend schema @link(url: "https://bad.horse", as: "horse")"#;
        with_matcher_for_schema(schema, |matcher| {
            assert!(matcher.is_key("key"));
            assert!(matcher.is_shareable("shareable"));
            assert!(!matcher.is_key("federation__key"));
            assert!(!matcher.is_shareable("federation__shareable"));
            assert!(!matcher.is_key("@key"));
        });
    }

    #[test]
    fn alias() {
        let schema = r#"extend schema @link(url: "https://specs.apollo.dev/federation/v2.3", as: "romulans")"#;
        with_matcher_for_schema(schema, |matcher| {
            assert!(!matcher.is_key("federation__key"));
            assert!(matcher.is_key("romulans__key"));
            assert!(!matcher.is_shareable("federation__shareable"));
            assert!(!matcher.is_shareable("@federation__shareable"));
            assert!(matcher.is_shareable("romulans__shareable"));
            assert!(!matcher.is_key("key"));
            assert!(!matcher.is_key("@key"));
            assert!(!matcher.is_shareable("shareable"));
        });
    }

    #[test]
    fn direct_import_and_alias() {
        let schema = r#"
            extend schema @link(
                url: "https://specs.apollo.dev/federation/v2.3",
                as: "romulans"
                import: [{ name: "@shareable", as: "partageable" }]
            )
        "#;
        with_matcher_for_schema(schema, |matcher| {
            assert!(!matcher.is_key("federation__key"));
            assert!(!matcher.is_shareable("romulans__shareable"));
            assert!(!matcher.is_shareable("romulans__partageable"));
            assert!(!matcher.is_shareable("romulans__shareable"));
            assert!(!matcher.is_shareable("@federation__shareable"));
            assert!(!matcher.is_key("key"));

            assert!(matcher.is_key("romulans__key"));
            assert!(matcher.is_shareable("partageable"));
        });
    }

    #[test]
    fn regular_imports() {
        let schema = r#"
            extend schema @link(
                url: "https://specs.apollo.dev/federation/v2.3",
                as: "romulans"
                import: [{ name: "@key" }, "@shareable"]
            )
        "#;
        with_matcher_for_schema(schema, |matcher| {
            assert!(!matcher.is_key("federation__key"));
            assert!(!matcher.is_shareable("federation__shareable"));
            assert!(!matcher.is_shareable("romulans__shareable"));
            assert!(!matcher.is_shareable("@federation__shareable"));

            assert!(matcher.is_key("key"));
            assert!(matcher.is_shareable("shareable"));
        });
    }
}
