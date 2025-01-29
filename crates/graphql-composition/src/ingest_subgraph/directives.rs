mod authorized;
mod consts;
mod match_name;

pub(super) use match_name::*;

use cynic_parser_deser::ConstDeserializer;
use graphql_federated_graph::directives::{CostDirective, DeprecatedDirective, ListSizeDirective};

use self::consts::*;
use super::*;

pub(super) fn ingest_directives(
    ctx: &mut Context<'_>,
    directive_site_id: DirectiveSiteId,
    directives_node: ast::iter::Iter<'_, ast::Directive<'_>>,
    location: impl Fn(&mut Subgraphs) -> String,
) {
    for directive in directives_node {
        let (directive_name_id, match_result) = match_directive_name(ctx, directive.name());

        dbg!(directive.name());
        dbg!(match_result);

        match match_result {
            DirectiveNameMatch::NoMatch
            | DirectiveNameMatch::Imported(_)
            | DirectiveNameMatch::Qualified(_, _)
            | DirectiveNameMatch::ComposeDirective
            | DirectiveNameMatch::Key => (),
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
        }

        // FIXME: should happen in composition, not in ingestion. GB-8398.
        if ctx.subgraphs.is_composed_directive(directive_name_id) {
            let arguments = directive
                .arguments()
                .map(|argument| {
                    (
                        ctx.subgraphs.strings.intern(argument.name()),
                        ast_value_to_subgraph_value(argument.value(), ctx.subgraphs),
                    )
                })
                .collect();
            ctx.subgraphs
                .insert_composed_directive_instance(directive_site_id, directive.name(), arguments);
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

        if let DirectiveNameMatch::Key = match_result {
            let fields_arg = directive.argument("fields").and_then(|v| v.value().as_str());
            let Some(fields_arg) = fields_arg else {
                continue;
            };
            let is_resolvable = directive
                .argument("resolvable")
                .and_then(|v| v.value().as_bool())
                .unwrap_or(true); // defaults to true
            ctx.subgraphs.push_key(definition_id, fields_arg, is_resolvable).ok();
        }
    }
}
