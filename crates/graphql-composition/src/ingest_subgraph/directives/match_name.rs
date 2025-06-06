use crate::subgraphs::StringId;

use super::*;

/// This function is the source of truth for matching directives by name when ingesting a
/// subgraph's GraphQL SDL.
///
/// The names of federation and other directives are influenced by `@link` directives on schema definitions
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
pub(in crate::ingest_subgraph) fn match_directive_name(
    ctx: &mut Context<'_>,
    directive_name: &str,
) -> (StringId, DirectiveNameMatch) {
    let (namespace, directive_name) = directive_name
        .split_once("__")
        .map(|(namespace, name)| {
            let namespace = ctx.subgraphs.strings.intern(namespace);
            (Some(namespace), name)
        })
        .unwrap_or((None, directive_name));

    let linked_schema_id =
        namespace.and_then(|namespace_str| ctx.subgraphs.get_linked_schema(ctx.subgraph_id, namespace_str));

    let directive_name_id = ctx.subgraphs.strings.intern(directive_name);

    let matched = match_directive_name_inner(ctx, directive_name_id, linked_schema_id, directive_name);

    (directive_name_id, matched)
}

fn match_directive_name_inner(
    ctx: &mut Context<'_>,
    directive_name_id: StringId,
    linked_schema_id: Option<subgraphs::LinkedSchemaId>,
    directive_name: &str,
) -> DirectiveNameMatch {
    if let Some(linked_schema_id) = linked_schema_id {
        if ctx.subgraphs.at(linked_schema_id).is_federation_v2(ctx.subgraphs) {
            return match_federation_directive_by_original_name(directive_name);
        }

        if ctx.subgraphs.at(linked_schema_id).is_composite_schemas(ctx.subgraphs) {
            return match_composite_schemas_directive_by_original_name(directive_name);
        }

        return DirectiveNameMatch::Qualified {
            linked_schema_id,
            directive_unqualified_name: directive_name_id,
        };
    }

    if let Some(imported_definition_id) = ctx
        .subgraphs
        .get_imported_definition(ctx.subgraph_id, directive_name_id)
    {
        let imported_definition = ctx.subgraphs.at(imported_definition_id);
        let linked_schema = ctx.subgraphs.at(imported_definition.linked_schema_id);

        if linked_schema.is_federation_v2(ctx.subgraphs) {
            let original_name = &ctx.subgraphs.strings.resolve(imported_definition.original_name);
            return match_federation_directive_by_original_name(original_name);
        }

        if linked_schema.is_composite_schemas(ctx.subgraphs) {
            let original_name = &ctx.subgraphs.strings.resolve(imported_definition.original_name);
            return match_composite_schemas_directive_by_original_name(original_name);
        }

        return DirectiveNameMatch::Imported {
            linked_definition_id: imported_definition_id,
        };
    }

    match directive_name {
        // Built-ins
        LINK => return DirectiveNameMatch::Link,
        "deprecated" => return DirectiveNameMatch::Deprecated,
        "specifiedBy" => return DirectiveNameMatch::SpecifiedBy,
        "oneOf" => return DirectiveNameMatch::OneOf,
        _ => (),
    }

    let federation_schema_has_been_linked = ctx.subgraphs.subgraph_links_federation_v2(ctx.subgraph_id);
    let is_virtual_subgraph = ctx.subgraphs.at(ctx.subgraph_id).is_virtual();

    // We auto-import federation directives only in cases where the federation spec hasn't been `@link`ed, to match federation v1 behaviour, or when the subgraph is virtual, because there is no legacy use case there, these are a new Grafbase feature.
    if !federation_schema_has_been_linked && !is_virtual_subgraph {
        return match_federation_directive_by_original_name(directive_name);
    }

    DirectiveNameMatch::NoMatch
}

fn match_composite_schemas_directive_by_original_name(original_name: &str) -> DirectiveNameMatch {
    match original_name {
        LOOKUP => DirectiveNameMatch::Lookup,
        KEY => DirectiveNameMatch::KeyFromCompositeSchemas,
        REQUIRE => DirectiveNameMatch::Require,
        SHAREABLE => DirectiveNameMatch::Shareable,
        IS => DirectiveNameMatch::Is,
        DERIVE => DirectiveNameMatch::Derive,
        EXTERNAL => DirectiveNameMatch::External,
        OVERRIDE => DirectiveNameMatch::Override,
        PROVIDES => DirectiveNameMatch::Provides,
        INACCESSIBLE => DirectiveNameMatch::Inaccessible,
        INTERNAL => DirectiveNameMatch::Internal,
        _ => DirectiveNameMatch::NoMatch,
    }
}

fn match_federation_directive_by_original_name(original_name: &str) -> DirectiveNameMatch {
    match original_name {
        AUTHENTICATED => DirectiveNameMatch::Authenticated,
        COMPOSE_DIRECTIVE => DirectiveNameMatch::ComposeDirective,
        COST => DirectiveNameMatch::Cost,
        EXTERNAL => DirectiveNameMatch::External,
        INACCESSIBLE => DirectiveNameMatch::Inaccessible,
        INTERFACE_OBJECT => DirectiveNameMatch::InterfaceObject,
        KEY => DirectiveNameMatch::Key,
        LIST_SIZE => DirectiveNameMatch::ListSize,
        OVERRIDE => DirectiveNameMatch::Override,
        POLICY => DirectiveNameMatch::Policy,
        PROVIDES => DirectiveNameMatch::Provides,
        REQUIRES => DirectiveNameMatch::Requires,
        REQUIRES_SCOPES => DirectiveNameMatch::RequiresScopes,
        SHAREABLE => DirectiveNameMatch::Shareable,
        TAG => DirectiveNameMatch::Tag,
        _ => DirectiveNameMatch::NoMatch,
    }
}

#[derive(Debug, Clone, Copy)]
pub(in crate::ingest_subgraph) enum DirectiveNameMatch {
    NoMatch,
    Qualified {
        linked_schema_id: subgraphs::LinkedSchemaId,
        directive_unqualified_name: StringId,
    },
    Imported {
        linked_definition_id: subgraphs::LinkedDefinitionId,
    },

    // GraphQL built-ins
    Deprecated,
    SpecifiedBy,

    // Composite schemas built-ins
    Lookup,
    Derive,
    Require,
    Is,
    KeyFromCompositeSchemas,
    Internal,

    // Federation built-ins
    Authenticated,
    ComposeDirective,
    Cost,
    External,
    Inaccessible,
    OneOf,
    InterfaceObject,
    Key,
    Link,
    ListSize,
    Override,
    Policy,
    Provides,
    Requires,
    RequiresScopes,
    Shareable,
    Tag,
}
