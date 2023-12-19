use super::*;

pub(super) fn merge_root_fields(ctx: &mut Context<'_>) {
    let mut query_types = Vec::new();
    let mut mutation_types = Vec::new();
    let mut subscription_types = Vec::new();

    for subgraph in ctx.subgraphs.iter_subgraphs() {
        if let Some(query_type) = subgraph.query_type() {
            query_types.push(query_type);
        }

        if let Some(mutation_type) = subgraph.mutation_type() {
            mutation_types.push(mutation_type);
        }

        if let Some(subscription_type) = subgraph.subscription_type() {
            subscription_types.push(subscription_type);
        }
    }

    let Some(query_id) = merge_fields("Query", &query_types, ctx) else {
        ctx.diagnostics
            .push_fatal("The root `Query` object is not defined in any subgraph.".to_owned());
        return;
    };

    ctx.set_query(query_id);

    if let Some(mutation_id) = merge_fields("Mutation", &mutation_types, ctx) {
        ctx.set_mutation(mutation_id);
    }

    if let Some(subscription_id) = merge_fields("Subscription", &subscription_types, ctx) {
        ctx.set_subscription(subscription_id);
    }
}

fn merge_fields<'a>(
    root: &'static str,
    definitions: &[subgraphs::DefinitionWalker<'a>],
    ctx: &mut Context<'a>,
) -> Option<federated::ObjectId> {
    if definitions.is_empty() {
        return None;
    }

    let type_name = ctx.insert_static_str(root);
    let directives = collect_composed_directives(definitions.iter().map(|def| def.directives()), ctx);

    let object_id = ctx.insert_object(type_name, None, directives);

    super::fields::for_each_field_group(definitions, |fields| {
        let Some(first) = fields.first() else { return };
        object::compose_object_fields(object_id, false, *first, fields, ctx);
    });

    Some(object_id)
}
