mod link;

use super::*;

pub(super) fn ingest_schema_definitions(ctx: &mut Context<'_>) {
    let schema_definitions = ctx.document.definitions().filter_map(|definition| match definition {
        ast::Definition::Schema(schema_definition) => Some(schema_definition),
        ast::Definition::SchemaExtension(schema_extension) => Some(schema_extension),
        _ => None,
    });

    let mut matcher = RootTypeMatcher::default();

    for schema_definition in schema_definitions {
        matcher.query = matcher.query.or(schema_definition
            .root_query_definition()
            .as_ref()
            .map(|query| query.named_type()));
        matcher.mutation = matcher.mutation.or(schema_definition
            .root_mutation_definition()
            .as_ref()
            .map(|mutation| mutation.named_type()));
        matcher.subscription = matcher.subscription.or(schema_definition
            .root_subscription_definition()
            .as_ref()
            .map(|subscription| subscription.named_type()));

        for directive in schema_definition.directives() {
            if directive.name() == "link" {
                link::ingest_link_directive(directive, ctx.subgraph_id, ctx.subgraphs);
            }
        }
    }

    ctx.root_type_matcher = matcher;

    // We must iterate a second time, because the complete first pass is necessary to have ingested all `@link`s, so we can match `@composeDirective`.
    for schema_definition in ctx.document.definitions().filter_map(|definition| match definition {
        ast::Definition::Schema(schema_definition) => Some(schema_definition),
        ast::Definition::SchemaExtension(schema_extension) => Some(schema_extension),
        _ => None,
    }) {
        for directive in schema_definition.directives() {
            let (_directive_name_id, match_result) = match_directive_name(ctx, directive.name());

            if let DirectiveNameMatch::ComposeDirective = match_result {
                for arg in directive.arguments() {
                    if arg.name() == "name" {
                        let Some(name) = arg.value().as_str() else {
                            ctx.subgraphs.push_ingestion_diagnostic(
                                ctx.subgraph_id,
                                "Invalid `@composeDirective` directive: `name` argument must be a string".to_owned(),
                            );
                            continue;
                        };

                        if !name.starts_with('@') {
                            ctx.subgraphs.push_ingestion_diagnostic(
                                ctx.subgraph_id,
                                "Invalid `@composeDirective` directive: `name` argument must start with `@`".to_owned(),
                            );

                            continue;
                        }

                        let name = name.trim_start_matches('@');
                        let (name, _) = match_directive_name(ctx, name);

                        ctx.subgraphs.insert_composed_directive(ctx.subgraph_id, name);
                    }
                }
            }
        }
    }

    // We now need a third pass to ingest the remaining directives
    // We must iterate a second time, because the complete first pass is necessary to have ingested all `@link`s, so we can match `@composeDirective`.
    for schema_definition in ctx.document.definitions().filter_map(|definition| match definition {
        ast::Definition::Schema(schema_definition) => Some(schema_definition),
        ast::Definition::SchemaExtension(schema_extension) => Some(schema_extension),
        _ => None,
    }) {
        for directive in schema_definition.directives() {
            let (directive_name_id, match_result) = match_directive_name(ctx, directive.name());

            match match_result {
                DirectiveNameMatch::Qualified {
                    linked_schema_id,
                    directive_unqualified_name,
                } => {
                    let arguments = ctx.ingest_extra_directive_arguments(directive.arguments());

                    ctx.subgraphs.push_extra_directive_on_schema_definition(
                        ctx.subgraph_id,
                        subgraphs::ExtraDirectiveRecord {
                            directive_site_id: DirectiveSiteId::from(0usize),
                            name: directive_unqualified_name,
                            arguments,
                            provenance: subgraphs::DirectiveProvenance::Linked {
                                linked_schema_id,
                                is_composed_directive: ctx
                                    .subgraphs
                                    .is_composed_directive(ctx.subgraph_id, directive_name_id),
                            },
                        },
                    );
                }
                DirectiveNameMatch::Imported { linked_definition_id } => {
                    let arguments = ctx.ingest_extra_directive_arguments(directive.arguments());
                    let linked_definition = ctx.subgraphs.at(linked_definition_id);

                    ctx.subgraphs.push_extra_directive_on_schema_definition(
                        ctx.subgraph_id,
                        subgraphs::ExtraDirectiveRecord {
                            directive_site_id: DirectiveSiteId::from(0usize),
                            name: linked_definition.original_name,
                            arguments,
                            provenance: subgraphs::DirectiveProvenance::Linked {
                                linked_schema_id: linked_definition.linked_schema_id,
                                is_composed_directive: ctx
                                    .subgraphs
                                    .is_composed_directive(ctx.subgraph_id, directive_name_id),
                            },
                        },
                    );
                }

                DirectiveNameMatch::NoMatch => {
                    let directive_name = ctx.subgraphs.at(directive_name_id);
                    ctx.subgraphs.push_ingestion_warning(
                        ctx.subgraph_id,
                        format!(
                            "Unknown directive `@{}` on schema definition or extension.",
                            directive_name.as_ref()
                        ),
                    );
                }

                _ => (),
            }
        }
    }
}

#[derive(Default)]
pub(super) struct RootTypeMatcher<'a> {
    query: Option<&'a str>,
    mutation: Option<&'a str>,
    subscription: Option<&'a str>,
}

impl RootTypeMatcher<'_> {
    pub(crate) fn is_query(&self, name: &str) -> bool {
        matches!(self.match_name(name), RootTypeMatch::Query)
    }

    pub(crate) fn match_name(&self, name: &str) -> RootTypeMatch {
        for (name_from_definition, default_name, match_case) in [
            (self.query, "Query", RootTypeMatch::Query),
            (self.mutation, "Mutation", RootTypeMatch::Mutation),
            (self.subscription, "Subscription", RootTypeMatch::Subscription),
        ] {
            match name_from_definition {
                Some(root_name) if root_name == name => return match_case,
                None if name == default_name => return match_case,

                Some(_) if name == default_name => return RootTypeMatch::NotRootButHasDefaultRootName,
                Some(_) | None => continue,
            }
        }

        RootTypeMatch::NotRoot
    }
}

pub(crate) enum RootTypeMatch {
    Query,
    Mutation,
    Subscription,
    NotRootButHasDefaultRootName,
    NotRoot,
}
