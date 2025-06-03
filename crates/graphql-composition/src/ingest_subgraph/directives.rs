mod authorized;
mod consts;
mod match_name;

pub(super) use match_name::*;

use self::consts::*;
use super::*;
use crate::composition_ir as ir;
use cynic_parser_deser::ConstDeserializer;
use graphql_federated_graph::directives::{CostDirective, DeprecatedDirective, ListSizeDirective};

pub(super) fn ingest_directives(
    ctx: &mut Context<'_>,
    directive_site_id: DirectiveSiteId,
    directives_node: ast::iter::Iter<'_, ast::Directive<'_>>,
    location: impl Fn(&mut Subgraphs) -> String,
) {
    for directive in directives_node {
        let (directive_name_id, match_result) = match_directive_name(ctx, directive.name());

        let is_composed_directive = ctx.subgraphs.is_composed_directive(ctx.subgraph_id, directive_name_id);

        match match_result {
            DirectiveNameMatch::NoMatch if is_composed_directive => {
                let arguments = ctx.ingest_extra_directive_arguments(directive.arguments());

                let record = subgraphs::ExtraDirectiveRecord {
                    directive_site_id,
                    name: directive_name_id,
                    arguments,
                    provenance: subgraphs::DirectiveProvenance::ComposedDirective,
                };

                ctx.subgraphs.push_directive(record);
            }

            DirectiveNameMatch::Imported { linked_definition_id } => {
                let arguments = ctx.ingest_extra_directive_arguments(directive.arguments());

                let linked_definition = ctx.subgraphs.at(linked_definition_id);

                let record = subgraphs::ExtraDirectiveRecord {
                    directive_site_id,
                    name: linked_definition.original_name,
                    arguments,
                    provenance: subgraphs::DirectiveProvenance::Linked {
                        linked_schema_id: linked_definition.linked_schema_id,
                        is_composed_directive,
                    },
                };

                ctx.subgraphs.push_directive(record);
            }

            DirectiveNameMatch::Qualified {
                linked_schema_id,
                directive_unqualified_name,
            } => {
                let arguments = ctx.ingest_extra_directive_arguments(directive.arguments());

                let record = subgraphs::ExtraDirectiveRecord {
                    directive_site_id,
                    name: directive_unqualified_name,
                    arguments,
                    provenance: subgraphs::DirectiveProvenance::Linked {
                        linked_schema_id,
                        is_composed_directive,
                    },
                };

                ctx.subgraphs.push_directive(record);
            }

            DirectiveNameMatch::Authorized => {
                if let Err(err) = authorized::ingest(directive_site_id, directive, ctx.subgraphs) {
                    let location = location(ctx.subgraphs);
                    ctx.subgraphs.push_ingestion_diagnostic(
                        ctx.subgraph_id,
                        format!("Error validating the @authorized directive at {location}: {err}",),
                    );
                };
            }
            DirectiveNameMatch::Cost => match directive.deserialize::<CostDirective>() {
                Ok(cost) => {
                    ctx.subgraphs.set_cost(directive_site_id, cost.weight);
                }
                Err(error) => {
                    let location = location(ctx.subgraphs);
                    ctx.subgraphs.push_ingestion_diagnostic(
                        ctx.subgraph_id,
                        format!("Error validating the @cost directive at {location}: {error}"),
                    );
                }
            },
            DirectiveNameMatch::ListSize => match directive.deserialize::<ListSizeDirective>() {
                Ok(directive) => {
                    ctx.subgraphs.set_list_size(directive_site_id, directive);
                }
                Err(error) => {
                    let location = location(ctx.subgraphs);
                    ctx.subgraphs.push_ingestion_diagnostic(
                        ctx.subgraph_id,
                        format!("Error validating the @listSize directive at {location}: {error}"),
                    );
                }
            },
            DirectiveNameMatch::Authenticated => {
                ctx.subgraphs.insert_authenticated(directive_site_id);
            }
            DirectiveNameMatch::Deprecated => match directive.deserialize::<DeprecatedDirective<'_>>() {
                Ok(directive) => ctx.subgraphs.insert_deprecated(directive_site_id, directive.reason),
                Err(err) => {
                    let location = location(ctx.subgraphs);
                    ctx.subgraphs.push_ingestion_diagnostic(
                        ctx.subgraph_id,
                        format!("Error validating the @deprecated directive at {location}: {err}",),
                    );
                }
            },
            DirectiveNameMatch::External => {
                ctx.subgraphs.set_external(directive_site_id);
            }
            DirectiveNameMatch::Inaccessible => {
                ctx.subgraphs.set_inaccessible(directive_site_id);
            }
            DirectiveNameMatch::Internal => {
                ctx.subgraphs.push_ir_directive(
                    directive_site_id,
                    ir::Directive::CompositeInternal(ctx.subgraph_id.idx().into()),
                );
            }
            DirectiveNameMatch::OneOf => {
                ctx.subgraphs.set_one_of(directive_site_id);
            }
            DirectiveNameMatch::InterfaceObject => {
                ctx.subgraphs.set_interface_object(directive_site_id);
            }
            DirectiveNameMatch::Override => {
                let from = directive
                    .argument("from")
                    .and_then(|v| v.value().as_str())
                    .map(|s| ctx.subgraphs.strings.intern(s));

                let label = directive
                    .argument("label")
                    .and_then(|v| v.value().as_str())
                    .map(|s| ctx.subgraphs.strings.intern(s));

                let Some(from) = from else { continue };

                ctx.subgraphs
                    .set_override(directive_site_id, subgraphs::OverrideDirective { from, label });
            }
            DirectiveNameMatch::Policy => {
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
                                ConstValue::String(string) => Some(ctx.subgraphs.strings.intern(string.as_str())),
                                _ => None,
                            })
                            .collect(),
                        _ => vec![],
                    };
                    ctx.subgraphs.insert_policy(directive_site_id, inner_policies);
                }
            }
            DirectiveNameMatch::Provides => {
                let fields_arg = directive.argument("fields").and_then(|arg| arg.value().as_str());
                let Some(fields_arg) = fields_arg else {
                    continue;
                };
                if let Err(err) = ctx.subgraphs.insert_provides(directive_site_id, fields_arg) {
                    ctx.subgraphs
                        .push_ingestion_diagnostic(ctx.subgraph_id, err.to_string());
                }
            }
            DirectiveNameMatch::Requires => {
                let fields_arg = directive.argument("fields").and_then(|arg| arg.value().as_str());

                let Some(fields_arg) = fields_arg else {
                    continue;
                };

                if let Err(err) = ctx.subgraphs.insert_requires(directive_site_id, fields_arg) {
                    ctx.subgraphs
                        .push_ingestion_diagnostic(ctx.subgraph_id, err.to_string());
                };
            }
            DirectiveNameMatch::RequiresScopes => {
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
                                ConstValue::String(string) => Some(ctx.subgraphs.strings.intern(string.as_str())),
                                _ => None,
                            })
                            .collect(),
                        _ => vec![],
                    };
                    ctx.subgraphs.append_required_scopes(directive_site_id, inner_scopes);
                }
            }
            DirectiveNameMatch::Shareable => {
                ctx.subgraphs.set_shareable(directive_site_id);
            }
            DirectiveNameMatch::Tag => {
                let Some(argument) = directive.argument("name") else {
                    continue;
                };

                if let Some(s) = argument.value().as_str() {
                    ctx.subgraphs.insert_tag(directive_site_id, s);
                }
            }

            DirectiveNameMatch::Lookup => {
                ctx.subgraphs.push_ir_directive(
                    directive_site_id,
                    crate::composition_ir::Directive::CompositeLookup(ctx.subgraph_id.idx().into()),
                );
            }
            DirectiveNameMatch::Derive => {
                ctx.subgraphs.push_ir_directive(
                    directive_site_id,
                    crate::composition_ir::Directive::CompositeDerive(ctx.subgraph_id.idx().into()),
                );
            }
            DirectiveNameMatch::Require => {
                let directive: graphql_federated_graph::directives::RequireDirective<'_> = match directive.deserialize()
                {
                    Ok(directive) => directive,
                    Err(err) => {
                        ctx.subgraphs
                            .push_ingestion_diagnostic(ctx.subgraph_id, err.to_string());
                        continue;
                    }
                };

                let field = ctx.subgraphs.strings.intern(directive.field);

                ctx.subgraphs.push_ir_directive(
                    directive_site_id,
                    crate::composition_ir::Directive::CompositeRequire {
                        subgraph_id: ctx.subgraph_id.idx().into(),
                        field,
                    },
                )
            }
            DirectiveNameMatch::Is => {
                let directive: graphql_federated_graph::directives::RequireDirective<'_> = match directive.deserialize()
                {
                    Ok(directive) => directive,
                    Err(err) => {
                        ctx.subgraphs
                            .push_ingestion_diagnostic(ctx.subgraph_id, err.to_string());
                        continue;
                    }
                };

                let field = ctx.subgraphs.strings.intern(directive.field);

                ctx.subgraphs.push_ir_directive(
                    directive_site_id,
                    crate::composition_ir::Directive::CompositeIs {
                        subgraph_id: ctx.subgraph_id.idx().into(),
                        field,
                    },
                )
            }

            DirectiveNameMatch::NoMatch => {
                let location = location(ctx.subgraphs);
                let directive_name = ctx.subgraphs.at(directive_name_id);

                ctx.subgraphs.push_ingestion_warning(
                    ctx.subgraph_id,
                    format!("Unknown directive `@{}` at `{}`", directive_name.as_ref(), location,),
                );
            }

            DirectiveNameMatch::ComposeDirective
            | DirectiveNameMatch::Key
            | DirectiveNameMatch::KeyFromCompositeSchemas
            | DirectiveNameMatch::Link
            | DirectiveNameMatch::SpecifiedBy => (),
        }
    }
}

pub(super) fn ingest_keys(
    definition_id: DefinitionId,
    directives_node: ast::iter::Iter<'_, ast::Directive<'_>>,
    ctx: &mut Context<'_>,
) {
    for directive in directives_node {
        let directive_name = directive.name();
        let (_, match_result) = match_directive_name(ctx, directive_name);

        if let DirectiveNameMatch::Key | DirectiveNameMatch::KeyFromCompositeSchemas = match_result {
            let fields_arg = directive.argument("fields").and_then(|v| v.value().as_str());
            let Some(fields_arg) = fields_arg else {
                continue;
            };

            // `resolvable` must be set to false for keys from the composite schema spec, because there is no _entities root resolver for them, they have `@lookup` fields instead.
            let is_resolvable = matches!(match_result, DirectiveNameMatch::Key)
                && directive
                    .argument("resolvable")
                    .and_then(|v| v.value().as_bool())
                    .unwrap_or(true); // defaults to true

            ctx.subgraphs.push_key(definition_id, fields_arg, is_resolvable).ok();
        }
    }
}
