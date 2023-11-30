use super::*;

pub(super) fn collect_composed_directives<'a>(
    containers: impl Iterator<Item = subgraphs::DirectiveContainerWalker<'a>> + Clone,
    ctx: &mut ComposeContext<'a>,
) -> Vec<federated::Directive> {
    let mut tags: BTreeSet<StringId> = BTreeSet::new();
    let mut is_inaccessible = false;
    let mut extra_directives = BTreeSet::new();
    let mut composed_directives = Vec::new();

    if let Some(deprecated) = containers.clone().find_map(|directives| directives.deprecated()) {
        composed_directives.push(federated::Directive {
            name: ctx.insert_static_str("deprecated"),
            arguments: deprecated
                .reason()
                .map(|reason| {
                    (
                        ctx.insert_static_str("reason"),
                        federated::Value::String(ctx.insert_string(reason.id)),
                    )
                })
                .map(|arg| vec![arg])
                .unwrap_or_default(),
        });
    }

    for container in containers {
        tags.extend(container.tags().map(|t| t.id));

        // The inaccessible directive is added whenever the item is inaccessible in any subgraph.
        is_inaccessible = is_inaccessible || container.inaccessible();

        for (name, arguments) in container.iter_composed_directives() {
            let name = ctx.insert_string(name);
            let arguments = arguments
                .iter()
                .map(|(name, value)| (ctx.insert_string(*name), subgraphs_value_to_federated_value(value, ctx)))
                .collect();

            extra_directives.insert(federated::Directive { name, arguments });
        }
    }

    if is_inaccessible {
        composed_directives.push(federated::Directive {
            name: ctx.insert_static_str("inaccessible"),
            arguments: Vec::new(),
        });
    }

    for tag in tags {
        let name = ctx.insert_string(tag);
        composed_directives.push(federated::Directive {
            name: ctx.insert_static_str("tag"),
            arguments: vec![(ctx.insert_static_str("name"), federated::Value::String(name))],
        });
    }

    composed_directives.extend(extra_directives);
    composed_directives
}

fn subgraphs_value_to_federated_value(value: &subgraphs::Value, ctx: &mut ComposeContext<'_>) -> federated::Value {
    match value {
        subgraphs::Value::String(value) => federated::Value::String(ctx.insert_string(*value)),
        subgraphs::Value::Int(value) => federated::Value::Int(*value),
        subgraphs::Value::Float(value) => federated::Value::Float(ctx.insert_string(*value)),
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
