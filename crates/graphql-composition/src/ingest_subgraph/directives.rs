mod authorized;
mod consts;

use cynic_parser::values::ConstList;
use cynic_parser_deser::ConstDeserializer;
use graphql_federated_graph::directives::{CostDirective, DeprecatedDirective, ListSizeDirective};

use self::consts::*;
use super::*;
use std::{borrow::Cow, collections::BTreeSet};

pub(super) fn ingest_directives(
    directive_site_id: DirectiveSiteId,
    directives_node: ast::iter::Iter<'_, ast::Directive<'_>>,
    subgraphs: &mut Subgraphs,
    directive_matcher: &DirectiveMatcher<'_>,
    subgraph: SubgraphId,
    location: impl Fn(&mut Subgraphs) -> String,
) {
    for directive in directives_node {
        let directive_name = directive.name();

        if !directive_matcher.is_known_directive(directive_name) {
            subgraphs.push_ingestion_warning(
                subgraph,
                format!(
                    "Unknown directive {directive_name} expected one of {}",
                    directive_matcher.iter_known_directives().collect::<Vec<_>>().join(", ")
                ),
            );
        }

        if directive_matcher.is_shareable(directive_name) {
            subgraphs.set_shareable(directive_site_id);
            continue;
        }

        if directive_matcher.is_external(directive_name) {
            subgraphs.set_external(directive_site_id);
            continue;
        }

        if directive_matcher.is_interface_object(directive_name) {
            subgraphs.set_interface_object(directive_site_id);
            continue;
        }

        if directive_matcher.is_inaccessible(directive_name) {
            subgraphs.set_inaccessible(directive_site_id);
            continue;
        }

        if directive_matcher.is_override(directive_name) {
            let from = directive
                .argument("from")
                .and_then(|v| v.value().as_str())
                .map(|s| subgraphs.strings.intern(s));

            let label = directive
                .argument("label")
                .and_then(|v| v.value().as_str())
                .map(|s| subgraphs.strings.intern(s));

            let Some(from) = from else { continue };

            subgraphs.set_override(directive_site_id, subgraphs::OverrideDirective { from, label });
            continue;
        }

        if directive_matcher.is_requires(directive_name) {
            let fields_arg = directive.argument("fields").and_then(|arg| arg.value().as_str());
            let Some(fields_arg) = fields_arg else {
                continue;
            };
            if let Err(err) = subgraphs.insert_requires(directive_site_id, fields_arg) {
                subgraphs.push_ingestion_diagnostic(subgraph, err.to_string());
            };
            continue;
        }

        if directive_matcher.is_provides(directive_name) {
            let fields_arg = directive.argument("fields").and_then(|arg| arg.value().as_str());
            let Some(fields_arg) = fields_arg else {
                continue;
            };
            if let Err(err) = subgraphs.insert_provides(directive_site_id, fields_arg) {
                subgraphs.push_ingestion_diagnostic(subgraph, err.to_string());
            }
            continue;
        }

        if directive_matcher.is_composed_directive(directive_name) {
            let arguments = directive
                .arguments()
                .map(|argument| {
                    (
                        subgraphs.strings.intern(argument.name()),
                        ast_value_to_subgraph_value(argument.value(), subgraphs),
                    )
                })
                .collect();
            subgraphs.insert_composed_directive_instance(directive_site_id, directive_name, arguments);
        }

        if directive_matcher.is_tag(directive_name) {
            let Some(argument) = directive.argument("name") else {
                continue;
            };

            if let Some(s) = argument.value().as_str() {
                subgraphs.insert_tag(directive_site_id, s);
            }
        }

        if directive_matcher.is_authenticated(directive_name) {
            subgraphs.insert_authenticated(directive_site_id);
        }

        if directive_matcher.is_requires_scope(directive_name) {
            let scopes = directive
                .argument("scopes")
                .into_iter()
                .flat_map(|scopes| scopes.value().as_items())
                .flatten();
            for scope in scopes {
                let inner_scopes: Vec<subgraphs::StringId> = match scope {
                    ConstValue::List(scopes) => scopes
                        .items()
                        .filter_map(|scope| match scope {
                            ConstValue::String(string) => Some(subgraphs.strings.intern(string.as_str())),
                            _ => None,
                        })
                        .collect(),
                    _ => vec![],
                };
                subgraphs.append_required_scopes(directive_site_id, inner_scopes);
            }
        }

        if directive_matcher.is_policy(directive_name) {
            let policies = directive
                .argument("policies")
                .into_iter()
                .flat_map(|scopes| scopes.value().as_items())
                .flatten();
            for policy in policies {
                let inner_policies: Vec<subgraphs::StringId> = match policy {
                    ConstValue::List(policies) => policies
                        .items()
                        .filter_map(|policy| match policy {
                            ConstValue::String(string) => Some(subgraphs.strings.intern(string.as_str())),
                            _ => None,
                        })
                        .collect(),
                    _ => vec![],
                };
                subgraphs.insert_policy(directive_site_id, inner_policies);
            }
        }

        if directive_name == "deprecated" {
            match directive.deserialize::<DeprecatedDirective<'_>>() {
                Ok(directive) => subgraphs.insert_deprecated(directive_site_id, directive.reason),
                Err(err) => {
                    let location = location(subgraphs);
                    subgraphs.push_ingestion_diagnostic(
                        subgraph,
                        format!("Error validating the @deprecated directive at {location}: {err}",),
                    );
                }
            }
        }

        if directive_matcher.is_authorized(directive_name) {
            if let Err(err) = authorized::ingest(directive_site_id, directive, subgraphs) {
                let location = location(subgraphs);
                subgraphs.push_ingestion_diagnostic(
                    subgraph,
                    format!("Error validating the @authorized directive at {location}: {err}",),
                );
            };
        }

        if directive_matcher.is_cost(directive_name) {
            match directive.deserialize::<CostDirective>() {
                Ok(cost) => {
                    subgraphs.set_cost(directive_site_id, cost.weight);
                }
                Err(error) => {
                    let location = location(subgraphs);
                    subgraphs.push_ingestion_diagnostic(
                        subgraph,
                        format!("Error validating the @cost directive at {location}: {error}"),
                    );
                }
            }
        }

        if directive_matcher.is_list_size(directive_name) {
            match directive.deserialize::<ListSizeDirective>() {
                Ok(directive) => {
                    subgraphs.set_list_size(directive_site_id, directive);
                }
                Err(error) => {
                    let location = location(subgraphs);
                    subgraphs.push_ingestion_diagnostic(
                        subgraph,
                        format!("Error validating the @listSize directive at {location}: {error}"),
                    );
                }
            }
        }
    }
}

pub(super) fn ingest_keys(
    definition_id: DefinitionId,
    directives_node: ast::iter::Iter<'_, ast::Directive<'_>>,
    subgraphs: &mut Subgraphs,
    directive_matcher: &DirectiveMatcher<'_>,
) {
    for directive in directives_node {
        let directive_name = directive.name();

        if directive_matcher.is_key(directive_name) {
            let fields_arg = directive.argument("fields").and_then(|v| v.value().as_str());
            let Some(fields_arg) = fields_arg else {
                continue;
            };
            let is_resolvable = directive
                .argument("resolvable")
                .and_then(|v| v.value().as_bool())
                .unwrap_or(true); // defaults to true
            subgraphs.push_key(definition_id, fields_arg, is_resolvable).ok();
        }
    }
}

pub(super) fn ingest_directive_definitions(
    document: &ast::TypeSystemDocument,
    mut push_error: impl FnMut(String),
) -> DirectiveMatcher<'_> {
    let schema_definition_directives = document
        .definitions()
        .filter_map(|def| match def {
            ast::Definition::Schema(schema_definition) => Some(schema_definition),
            ast::Definition::SchemaExtension(schema_definition) => Some(schema_definition),
            _ => None,
        })
        .flat_map(|definition| definition.directives());

    let mut directive_matcher = schema_definition_directives
        .clone()
        .find(|d| DirectiveMatcher::is_federation_directive(*d))
        .map(DirectiveMatcher::new)
        .unwrap_or_default();

    let mut composed_directives = BTreeSet::new();

    for name in schema_definition_directives
        .filter(|directive| directive_matcher.is_compose_directive(directive.name()))
        .filter_map(|directive| directive.argument("name"))
        .filter_map(|argument| argument.value().as_str())
    {
        composed_directives.insert(name.trim_start_matches('@'));

        if !name.starts_with('@') {
            push_error(format!(
                "The `{}` directive is missing the `@` prefix in @composeDirective.",
                name
            ));
        }
    }

    directive_matcher
        .all_known_directives
        .extend(composed_directives.iter().map(|directive| Cow::Borrowed(*directive)));

    directive_matcher.composed_directives = composed_directives;

    directive_matcher
        .all_known_directives
        .extend(document.definitions().filter_map(|def| match def {
            ast::Definition::Directive(def) => Some(Cow::Borrowed(def.name())),
            _ => None,
        }));

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
///     `@federation__`, for example `@federation__shareable`.
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
    cost: Cow<'a, str>,
    list_size: Cow<'a, str>,

    composed_directives: BTreeSet<&'a str>,

    all_known_directives: BTreeSet<Cow<'a, str>>,
}

const DEFAULT_FEDERATION_PREFIX: &str = "federation__";

impl Default for DirectiveMatcher<'_> {
    fn default() -> Self {
        let mut all_known_directives = default_known_directives();

        let mut record = |directive: &'static str| {
            let directive = Cow::Borrowed(directive);
            all_known_directives.insert(directive.clone());
            directive
        };

        DirectiveMatcher {
            authenticated: record(AUTHENTICATED),
            compose_directive: record(COMPOSE_DIRECTIVE),
            composed_directives: BTreeSet::new(),
            external: record(EXTERNAL),
            inaccessible: record(INACCESSIBLE),
            interface_object: record(INTERFACE_OBJECT),
            key: record(KEY),
            policy: record(POLICY),
            provides: record(PROVIDES),
            r#override: record(OVERRIDE),
            requires: record(REQUIRES),
            requires_scopes: record(REQUIRES_SCOPES),
            shareable: record(SHAREABLE),
            tag: record(TAG),
            cost: record(COST),
            list_size: record(LIST_SIZE),
            all_known_directives,
        }
    }
}

impl<'a> DirectiveMatcher<'a> {
    pub(crate) fn is_federation_directive(directive: ast::Directive<'_>) -> bool {
        if directive.name() != "link" {
            return false;
        }

        directive
            .argument("url")
            .map(|url| match url.value() {
                ConstValue::String(s) => s.value().contains("dev/federation/v2"),
                _ => false,
            })
            .unwrap_or_default()
    }

    /// Matcher for federation directives in a given subgraph. See [DirectiveMatcher] for more docs.
    pub(crate) fn new(directive: ast::Directive<'a>) -> DirectiveMatcher<'a> {
        let mut r#as = None;
        let mut imported: Vec<(&str, &str)> = Vec::new();

        for argument in directive.arguments() {
            match (argument.name(), argument.value()) {
                ("as", ConstValue::String(value)) => r#as = Some(value.as_str()),
                ("import", ConstValue::List(imports)) => read_imports(imports, &mut imported),
                _ => (),
            }
        }

        let mut all_known_directives = default_known_directives();

        let federation_prefix = r#as
            .map(|prefix| Cow::Owned(format!("{prefix}__")))
            .unwrap_or(Cow::Borrowed(DEFAULT_FEDERATION_PREFIX));
        let mut final_name = |directive_name: &str| {
            let name = imported
                .iter()
                .find(|(original, _alias)| *original == directive_name)
                .map(|(_, alias)| Cow::Borrowed(*alias))
                .unwrap_or_else(|| Cow::Owned(format!("{federation_prefix}{directive_name}")));

            all_known_directives.insert(name.clone());
            name
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
            cost: final_name(COST),
            list_size: final_name(LIST_SIZE),
            all_known_directives,
        }
    }

    pub(crate) fn is_authorized(&self, directive_name: &str) -> bool {
        directive_name == AUTHORIZED
    }

    pub(crate) fn is_compose_directive(&self, directive_name: &str) -> bool {
        self.compose_directive == directive_name
    }

    pub(crate) fn is_composed_directive(&self, directive_name: &str) -> bool {
        // The `@authorized` directive is an exception. Directives used in subgraph schemas are either built-in federation directives (`@requires`, `@key`, etc.) or custom, composed directives with `@composeDirective`. Since `@authorized` is not part of the federation spec, some frameworks like async-graphql (Rust) will produce an `@composeDirective` with the `@authorized` directive. We should not consider `@authorized` as a composed directive however, because that means we would emit it again.
        //
        // TODO: as for other imported composition directives, we should forbid their use in `@composeDirective`.
        if directive_name == AUTHORIZED {
            return false;
        }

        self.composed_directives
            .contains(&directive_name.trim_start_matches('@'))
    }

    pub(crate) fn iter_composed_directives(&self) -> impl Iterator<Item = &str> {
        self.composed_directives.iter().copied()
    }

    pub(crate) fn iter_known_directives(&self) -> impl Iterator<Item = &str> {
        self.all_known_directives.iter().map(|directive| directive.as_ref())
    }

    pub(crate) fn is_known_directive(&self, directive_name: &str) -> bool {
        self.all_known_directives.contains(directive_name)
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

    fn is_cost(&self, directive_name: &str) -> bool {
        self.cost == directive_name
    }

    fn is_list_size(&self, directive_name: &str) -> bool {
        self.list_size == directive_name
    }
}

fn default_known_directives<'a>() -> BTreeSet<Cow<'a, str>> {
    let mut hash_set = BTreeSet::new();
    hash_set.insert(Cow::Borrowed("authorized"));
    hash_set
}

fn read_imports<'a>(ast_imports: ConstList<'a>, out: &mut Vec<(&'a str, &'a str)>) {
    for import in ast_imports.items() {
        match import {
            ConstValue::String(import) => {
                let import = import.as_str().trim_start_matches('@');
                out.push((import, import));
            }
            ConstValue::Object(obj) => {
                if let Some(ConstValue::String(name)) = obj.get("name") {
                    let alias = obj.get("as").and_then(|value| match value {
                        ConstValue::String(s) => Some(s),
                        _ => None,
                    });
                    out.push((
                        name.as_str().trim_start_matches('@'),
                        alias.unwrap_or(name).as_str().trim_start_matches('@'),
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

    #[allow(clippy::panic)]
    fn with_matcher_for_schema(graphql_sdl: &str, test: impl FnOnce(DirectiveMatcher<'_>)) {
        let ast = cynic_parser::parse_type_system_document(graphql_sdl).unwrap();
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
