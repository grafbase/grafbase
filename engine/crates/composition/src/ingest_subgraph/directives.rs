mod consts;

use self::consts::*;
use super::*;
use std::{borrow::Cow, collections::BTreeSet};

pub(super) fn ingest_directives(
    directives: DirectiveSiteId,
    directives_node: &[Positioned<ast::ConstDirective>],
    subgraphs: &mut Subgraphs,
    directive_matcher: &DirectiveMatcher<'_>,
) {
    for directive in directives_node {
        let directive_name = &directive.node.name.node;
        if directive_matcher.is_shareable(directive_name) {
            subgraphs.set_shareable(directives);
            continue;
        }

        if directive_matcher.is_external(directive_name) {
            subgraphs.set_external(directives);
            continue;
        }

        if directive_matcher.is_interface_object(directive_name) {
            subgraphs.set_interface_object(directives);
            continue;
        }

        if directive_matcher.is_inaccessible(directive_name) {
            subgraphs.set_inaccessible(directives);
            continue;
        }

        if directive_matcher.is_override(directive_name) {
            let from = directive
                .node
                .get_argument("from")
                .and_then(|v| match &v.node {
                    ConstValue::String(s) => Some(s.as_str()),
                    _ => None,
                })
                .map(|s| subgraphs.strings.intern(s));

            let Some(from) = from else { continue };

            subgraphs.set_override(directives, from);
            continue;
        }

        if directive_matcher.is_requires(directive_name) {
            let fields_arg = directive.node.get_argument("fields").map(|v| &v.node);
            let Some(ConstValue::String(fields_arg)) = fields_arg else {
                continue;
            };
            subgraphs.insert_requires(directives, fields_arg).ok();
            continue;
        }

        if directive_matcher.is_provides(directive_name) {
            let fields_arg = directive.node.get_argument("fields").map(|v| &v.node);
            let Some(ConstValue::String(fields_arg)) = fields_arg else {
                continue;
            };
            subgraphs.insert_provides(directives, fields_arg).ok();
            continue;
        }

        if directive_matcher.is_composed_directive(directive_name) {
            let arguments = directive
                .node
                .arguments
                .iter()
                .map(|(name, value)| {
                    (
                        subgraphs.strings.intern(name.node.as_str()),
                        ast_value_to_subgraph_value(&value.node, subgraphs),
                    )
                })
                .collect();
            subgraphs.insert_composed_directive_instance(directives, directive_name.as_str(), arguments);
        }

        if directive_matcher.is_tag(directive_name) {
            let Some(value) = directive.node.get_argument("name") else {
                continue;
            };

            if let async_graphql_value::ConstValue::String(s) = &value.node {
                subgraphs.insert_tag(directives, s.as_str());
            }
        }

        if directive_matcher.is_authenticated(directive_name) {
            subgraphs.insert_authenticated(directives);
        }

        if directive_matcher.is_requires_scope(directive_name) {
            let scopes = directive
                .node
                .get_argument("scopes")
                .into_iter()
                .flat_map(|scopes| match &scopes.node {
                    ConstValue::List(list) => Some(list),
                    _ => None,
                })
                .flatten();
            for scope in scopes {
                let inner_scopes: Vec<subgraphs::StringId> = match scope {
                    ConstValue::List(scopes) => scopes
                        .iter()
                        .filter_map(|scope| match scope {
                            ConstValue::String(string) => Some(subgraphs.strings.intern(string.as_str())),
                            _ => None,
                        })
                        .collect(),
                    _ => vec![],
                };
                subgraphs.insert_requires_scopes(directives, inner_scopes);
            }
        }

        if directive_matcher.is_policy(directive_name) {
            let policies = directive
                .node
                .get_argument("policies")
                .into_iter()
                .flat_map(|scopes| match &scopes.node {
                    ConstValue::List(list) => Some(list),
                    _ => None,
                })
                .flatten();
            for policy in policies {
                let inner_policies: Vec<subgraphs::StringId> = match policy {
                    ConstValue::List(policies) => policies
                        .iter()
                        .filter_map(|policy| match policy {
                            ConstValue::String(string) => Some(subgraphs.strings.intern(string.as_str())),
                            _ => None,
                        })
                        .collect(),
                    _ => vec![],
                };
                subgraphs.insert_policy(directives, inner_policies);
            }
        }

        if directive_name == "deprecated" {
            let reason = directive.node.get_argument("reason").and_then(|v| match &v.node {
                async_graphql_value::ConstValue::String(s) => Some(s.as_str()),
                _ => None,
            });

            subgraphs.insert_deprecated(directives, reason);
        }
    }
}

pub(super) fn ingest_keys(
    definition_id: DefinitionId,
    directives_node: &[Positioned<ast::ConstDirective>],
    subgraphs: &mut Subgraphs,
    directive_matcher: &DirectiveMatcher<'_>,
) {
    for directive in directives_node {
        let directive_name = &directive.node.name.node;

        if directive_matcher.is_key(directive_name) {
            let fields_arg = directive.node.get_argument("fields").map(|v| &v.node);
            let Some(ConstValue::String(fields_arg)) = fields_arg else {
                continue;
            };
            let is_resolvable = directive
                .node
                .get_argument("resolvable")
                .and_then(|v| match v.node {
                    ConstValue::Boolean(b) => Some(b),
                    _ => None,
                })
                .unwrap_or(true); // defaults to true
            subgraphs.push_key(definition_id, fields_arg, is_resolvable).ok();
        }
    }
}

pub(super) fn ingest_directive_definitions(
    document: &ast::ServiceDocument,
    mut push_error: impl FnMut(String),
) -> DirectiveMatcher<'_> {
    let schema_definition_directives = document
        .definitions
        .iter()
        .filter_map(|def| {
            if let ast::TypeSystemDefinition::Schema(schema) = def {
                Some(schema)
            } else {
                None
            }
        })
        .flat_map(|definition| definition.node.directives.iter());

    let mut directive_matcher = schema_definition_directives
        .clone()
        .map(|d| &d.node)
        .find(|d| DirectiveMatcher::is_federation_directive(d))
        .map(DirectiveMatcher::new)
        .unwrap_or_default();

    let mut composed_directives = BTreeSet::new();

    for name in schema_definition_directives
        .filter(|directive| directive_matcher.is_compose_directive(directive.node.name.node.as_str()))
        .filter_map(|directive| directive.node.get_argument("name"))
        .filter_map(|directive_name| match &directive_name.node {
            ConstValue::String(s) => Some(s.as_str()),
            _ => None,
        })
    {
        composed_directives.insert(name.trim_start_matches('@'));

        if !name.starts_with('@') {
            push_error(format!(
                "The `{}` directive is missing the `@` prefix in @composeDirective.",
                name
            ));
        }
    }

    directive_matcher.composed_directives = composed_directives;

    directive_matcher
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
pub(crate) struct DirectiveMatcher<'a> {
    shareable: Cow<'a, str>,
    key: Cow<'a, str>,
    external: Cow<'a, str>,
    provides: Cow<'a, str>,
    requires: Cow<'a, str>,
    inaccessible: Cow<'a, str>,
    interface_object: Cow<'a, str>,
    r#override: Cow<'a, str>,
    compose_directive: Cow<'a, str>,
    requires_scopes: Cow<'a, str>,
    authenticated: Cow<'a, str>,
    policy: Cow<'a, str>,
    tag: Cow<'a, str>,

    /// directive name -> is repeatable
    ///
    /// The value is None wherever no definition was found for the directive.
    composed_directives: BTreeSet<&'a str>,
}

const DEFAULT_FEDERATION_PREFIX: &str = "federation__";

impl Default for DirectiveMatcher<'_> {
    fn default() -> Self {
        DirectiveMatcher {
            authenticated: Cow::Borrowed(AUTHENTICATED),
            compose_directive: Cow::Borrowed(COMPOSE_DIRECTIVE),
            composed_directives: BTreeSet::new(),
            external: Cow::Borrowed(EXTERNAL),
            inaccessible: Cow::Borrowed(INACCESSIBLE),
            interface_object: Cow::Borrowed(INTERFACE_OBJECT),
            key: Cow::Borrowed(KEY),
            policy: Cow::Borrowed(POLICY),
            provides: Cow::Borrowed(PROVIDES),
            r#override: Cow::Borrowed(OVERRIDE),
            requires: Cow::Borrowed(REQUIRES),
            requires_scopes: Cow::Borrowed(REQUIRES_SCOPES),
            shareable: Cow::Borrowed(SHAREABLE),
            tag: Cow::Borrowed(TAG),
        }
    }
}

impl<'a> DirectiveMatcher<'a> {
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

    /// Matcher for federation directives in a given subgraph. See [DirectiveMatcher] for more docs.        
    pub(crate) fn new(directive: &'a ast::ConstDirective) -> DirectiveMatcher<'a> {
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

        DirectiveMatcher {
            authenticated: final_name(AUTHENTICATED),
            compose_directive: final_name(COMPOSE_DIRECTIVE),
            composed_directives: BTreeSet::new(),
            external: final_name(EXTERNAL),
            inaccessible: final_name(INACCESSIBLE),
            interface_object: final_name(INTERFACE_OBJECT),
            key: final_name(KEY),
            policy: final_name(POLICY),
            provides: final_name(PROVIDES),
            r#override: final_name(OVERRIDE),
            requires: final_name(REQUIRES),
            requires_scopes: final_name(REQUIRES_SCOPES),
            shareable: final_name(SHAREABLE),
            tag: final_name(TAG),
        }
    }

    pub(crate) fn is_compose_directive(&self, directive_name: &str) -> bool {
        self.compose_directive == directive_name
    }

    pub(crate) fn is_composed_directive(&self, directive_name: &str) -> bool {
        self.composed_directives
            .contains(&directive_name.trim_start_matches('@'))
    }

    pub(crate) fn iter_composed_directives(&self) -> impl Iterator<Item = &str> {
        self.composed_directives.iter().copied()
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

    pub(crate) fn is_authenticated(&self, directive_name: &str) -> bool {
        self.authenticated == directive_name
    }

    pub(crate) fn is_policy(&self, directive_name: &str) -> bool {
        self.policy == directive_name
    }

    pub(crate) fn is_requires_scope(&self, directive_name: &str) -> bool {
        self.requires_scopes == directive_name
    }

    pub(crate) fn is_tag(&self, directive_name: &str) -> bool {
        self.tag == directive_name
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

fn ast_value_to_subgraph_value(value: &ConstValue, subgraphs: &mut Subgraphs) -> subgraphs::Value {
    match &value {
        ConstValue::Null | ConstValue::Binary(_) => unreachable!("null or bytes value in argument"),
        ConstValue::Number(n) if n.is_u64() || n.is_i64() => subgraphs::Value::Int(n.as_i64().unwrap()),
        ConstValue::Number(n) => subgraphs::Value::Float(n.as_f64().unwrap()),
        ConstValue::String(s) => subgraphs::Value::String(subgraphs.strings.intern(s.as_str())),
        ConstValue::Boolean(b) => subgraphs::Value::Boolean(*b),
        ConstValue::Enum(e) => subgraphs::Value::Enum(subgraphs.strings.intern(e.as_str())),
        ConstValue::List(l) => {
            subgraphs::Value::List(l.iter().map(|v| ast_value_to_subgraph_value(v, subgraphs)).collect())
        }
        ConstValue::Object(o) => subgraphs::Value::Object(
            o.iter()
                .map(|(k, v)| {
                    (
                        subgraphs.strings.intern(k.as_str()),
                        ast_value_to_subgraph_value(v, subgraphs),
                    )
                })
                .collect(),
        ),
    }
}

#[cfg(test)]
mod federation_directives_matcher_tests {
    use datatest_stable as _;
    use miette as _;
    use similar as _;

    use super::*;

    #[allow(clippy::panic)]
    fn with_matcher_for_schema(graphql_sdl: &str, test: impl FnOnce(DirectiveMatcher<'_>)) {
        let ast = async_graphql_parser::parse_schema(graphql_sdl).unwrap();
        let matcher = ingest_directive_definitions(&ast, |error| panic!("{error}"));
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
