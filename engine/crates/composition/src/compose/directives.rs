use super::*;

pub(super) fn collect_composed_directives<'a>(
    sites: impl Iterator<Item = subgraphs::DirectiveSiteWalker<'a>> + Clone,
    ctx: &mut ComposeContext<'_>,
) -> federated::Directives {
    let mut tags: BTreeSet<StringId> = BTreeSet::new();
    let mut is_inaccessible = false;
    let mut authenticated = false;
    let mut extra_directives = Vec::new();
    let mut ids: Option<federated::Directives> = None;
    let mut push_directive = |ctx: &mut ComposeContext<'_>, directive| {
        let id = ctx.insert_directive(directive);
        if let Some((_start, len)) = &mut ids {
            *len += 1;
        } else {
            ids = Some((id, 1));
        }
    };

    if let Some(deprecated) = sites.clone().find_map(|directives| directives.deprecated()) {
        let directive = federated::Directive::Deprecated {
            reason: deprecated.reason().map(|reason| ctx.insert_string(reason.id)),
        };
        push_directive(ctx, directive);
    }

    for site in sites {
        tags.extend(site.tags().map(|t| t.id));

        // The inaccessible directive is added whenever the item is inaccessible in any subgraph.
        is_inaccessible = is_inaccessible || site.inaccessible();

        // @authenticated behaves like @inaccessible
        authenticated = authenticated || site.authenticated();

        for (name, arguments) in site.iter_composed_directives() {
            let name = ctx.insert_string(name);
            let arguments = arguments
                .iter()
                .map(|(name, value)| (ctx.insert_string(*name), subgraphs_value_to_federated_value(value, ctx)))
                .collect();

            extra_directives.push(federated::Directive::Other { name, arguments });
        }
    }

    if is_inaccessible {
        push_directive(ctx, federated::Directive::Inaccessible);
    }

    if authenticated {
        push_directives(ctx, federated::Directive::Authenticated)
    }

    for tag in tags {
        let name = ctx.insert_string(tag);
        let directive = federated::Directive::Other {
            name: ctx.insert_static_str("tag"),
            arguments: vec![(ctx.insert_static_str("name"), federated::Value::String(name))],
        };
        push_directive(ctx, directive);
    }

    extra_directives.dedup();

    for directive in extra_directives {
        push_directive(ctx, directive)
    }

    ids.unwrap_or((federated::DirectiveId(0), 0))
}

fn subgraphs_value_to_federated_value(value: &subgraphs::Value, ctx: &mut ComposeContext<'_>) -> federated::Value {
    match value {
        subgraphs::Value::String(value) => federated::Value::String(ctx.insert_string(*value)),
        subgraphs::Value::Int(value) => federated::Value::Int(*value),
        subgraphs::Value::Float(value) => federated::Value::Float(*value),
        subgraphs::Value::Boolean(value) => federated::Value::Boolean(*value),
        subgraphs::Value::Enum(value) => federated::Value::EnumValue(ctx.insert_string(*value)),
        subgraphs::Value::Object(value) => federated::Value::Object(
            value
                .iter()
                .map(|(k, v)| (ctx.insert_string(*k), subgraphs_value_to_federated_value(v, ctx)))
                .collect(),
        ),
        subgraphs::Value::List(value) => federated::Value::List(
            value
                .iter()
                .map(|v| subgraphs_value_to_federated_value(v, ctx))
                .collect(),
        ),
    }
}
