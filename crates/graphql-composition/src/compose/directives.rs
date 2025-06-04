use super::*;

pub(super) fn create_join_type_from_definitions(
    definitions: &[DefinitionWalker<'_>],
) -> impl Iterator<Item = ir::Directive> {
    let mut subgraph_ids = definitions
        .iter()
        .map(|def| federated::SubgraphId::from(def.subgraph_id().idx()))
        .collect::<Vec<_>>();
    subgraph_ids.sort_unstable();
    subgraph_ids.into_iter().dedup().map(|subgraph_id| {
        ir::Directive::JoinType(ir::JoinTypeDirective {
            subgraph_id,
            key: None,
            is_interface_object: false,
        })
    })
}

pub(super) fn collect_composed_directives<'a>(
    sites: impl Iterator<Item = subgraphs::DirectiveSiteWalker<'a>> + Clone,
    ctx: &mut ComposeContext<'_>,
) -> Vec<ir::Directive> {
    let mut tags: BTreeSet<StringId> = BTreeSet::new();
    let mut is_inaccessible = false;
    let mut authenticated = false;
    let mut cost = None;
    let mut list_size = None;
    let mut extra_directives = Vec::new();
    let mut out = Vec::new();

    out.extend(
        sites
            .clone()
            .filter_map(|dir| dir.list_size().cloned())
            .reduce(|lhs, rhs| lhs.merge(rhs))
            .map(ir::Directive::ListSize),
    );

    if let Some(deprecated) = sites.clone().find_map(|directives| directives.deprecated()) {
        let directive = ir::Directive::Deprecated {
            reason: deprecated.reason().map(|reason| ctx.insert_string(reason.id)),
        };
        out.push(directive);
    }

    if sites.clone().any(|dirs| dirs.one_of()) {
        out.push(ir::Directive::OneOf);
    }

    for site in sites.clone() {
        tags.extend(site.tags().map(|t| t.id));

        // The directive is added whenever it's applied in any subgraph.
        is_inaccessible = is_inaccessible || site.inaccessible();
        authenticated = authenticated || site.authenticated();

        cost = cost.or(site.cost());
        list_size = list_size.or(site.list_size());

        for directive in site.iter_ir_directives() {
            extra_directives.push(directive.clone());
        }

        for directive in site.iter_extra_directives() {
            let provenance = match directive.provenance {
                subgraphs::DirectiveProvenance::ComposedDirective => Some(ir::DirectiveProvenance::ComposeDirective),
                subgraphs::DirectiveProvenance::Linked {
                    linked_schema_id,
                    is_composed_directive,
                } => match (
                    ctx.get_extension_for_linked_schema(linked_schema_id),
                    is_composed_directive,
                ) {
                    (Some(_), true) => {
                        ctx.diagnostics.push_fatal(String::from(
                            "Directives from extensions must not be composed with `@composeDirective`",
                        ));
                        None
                    }
                    (Some(extension_id), false) => {
                        ctx.mark_used_extension(extension_id);
                        Some(ir::DirectiveProvenance::LinkedFromExtension {
                            linked_schema_id,
                            extension_id,
                        })
                    }
                    (None, true) => Some(ir::DirectiveProvenance::ComposeDirective),
                    (None, false) => None,
                },
            };

            let Some(provenance) = provenance else {
                continue;
            };

            let name = ctx.insert_string(directive.name);

            let arguments = directive
                .arguments
                .iter()
                .map(|(name, value)| (ctx.insert_string(*name), value.clone()))
                .collect();

            extra_directives.push(ir::Directive::Other {
                provenance,
                name,
                arguments,
            });
        }
    }

    if is_inaccessible {
        out.push(ir::Directive::Inaccessible);
    }

    if authenticated {
        out.push(ir::Directive::Authenticated);
    }

    if let Some(weight) = cost {
        out.push(ir::Directive::Cost { weight });
    }

    // @requiresScopes
    {
        let mut scopes: Vec<Vec<federated::StringId>> = Vec::new();

        for scopes_arg in sites.clone().flat_map(|directives| directives.requires_scopes()) {
            scopes.push(
                scopes_arg
                    .iter()
                    .map(|scope| ctx.insert_string(*scope))
                    .collect::<Vec<_>>(),
            );
        }

        scopes.sort();
        scopes.dedup();

        if !scopes.is_empty() {
            out.push(ir::Directive::RequiresScopes(scopes));
        }
    }

    // @policy
    {
        let mut policies: Vec<Vec<federated::StringId>> = Vec::new();

        for policies_arg in sites.clone().flat_map(|directives| directives.policies()) {
            policies.push(
                policies_arg
                    .iter()
                    .map(|scope| ctx.insert_string(*scope))
                    .collect::<Vec<_>>(),
            );
        }

        policies.sort();
        policies.dedup();

        if !policies.is_empty() {
            out.push(ir::Directive::Policy(policies));
        }
    }

    for tag in tags {
        let directive = ir::Directive::Other {
            name: ctx.insert_static_str("tag"),
            arguments: vec![(ctx.insert_static_str("name"), subgraphs::Value::String(tag))],
            provenance: ir::DirectiveProvenance::Builtin,
        };
        out.push(directive);
    }

    extra_directives.dedup();

    for directive in extra_directives {
        out.push(directive);
    }

    out
}
