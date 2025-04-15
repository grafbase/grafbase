use std::mem::take;

use crate::{
    FieldResolverExtensionDefinitionRecord, ResolverDefinitionRecord, SelectionSetResolverExtensionDefinitionRecord,
    SubgraphId, VirtualSubgraphId, builder::GraphBuilder,
};

pub(crate) fn finalize_selection_set_resolvers(ctx: &mut GraphBuilder<'_>) -> Result<(), String> {
    // Ensure they're not mixed with field resolvers.
    for resolver in &ctx.graph.resolver_definitions {
        if let Some(FieldResolverExtensionDefinitionRecord { directive_id }) = resolver.as_field_resolver_extension() {
            let subgraph_id = ctx.graph[*directive_id]
                .subgraph_id
                .as_virtual()
                .expect("should have failed at directive creation");
            if let Some(id) = ctx.virtual_subgraph_to_selection_set_resolver[usize::from(subgraph_id)] {
                return Err(format!(
                    "Selection Set Resolver extension {} cannot be mixed with other resolvers in subgraph '{}', found {}",
                    ctx[id].manifest.id,
                    ctx[ctx.subgraphs[subgraph_id].subgraph_name_id],
                    ctx[ctx.graph[*directive_id].extension_id].manifest.id
                ));
            }
        }
    }

    let field_ids_list = {
        let mut list = vec![ctx.graph[ctx.graph.root_operation_types_record.query_id].field_ids];
        if let Some(mutation_id) = ctx.graph.root_operation_types_record.mutation_id {
            list.push(ctx.graph[mutation_id].field_ids);
        }
        if let Some(subscription_id) = ctx.graph.root_operation_types_record.subscription_id {
            list.push(ctx.graph[subscription_id].field_ids);
        }
        list
    };
    let mut resolver_definitions = take(&mut ctx.graph.resolver_definitions);
    for (ix, extension_id) in take(&mut ctx.virtual_subgraph_to_selection_set_resolver)
        .into_iter()
        .enumerate()
    {
        let Some(extension_id) = extension_id else {
            continue;
        };
        let virtual_subgraph_id = VirtualSubgraphId::from(ix);
        let subgraph_id = SubgraphId::from(virtual_subgraph_id);

        for field_ids in &field_ids_list {
            for field in &mut ctx.graph[*field_ids] {
                if field.exists_in_subgraph_ids.contains(&subgraph_id) {
                    // Each field has its dedicated resolvers and they don't support batching
                    // multiple fields for now.
                    resolver_definitions.push(ResolverDefinitionRecord::SelectionSetResolverExtension(
                        SelectionSetResolverExtensionDefinitionRecord {
                            subgraph_id: virtual_subgraph_id,
                            extension_id,
                        },
                    ));
                    field.resolver_ids.push((resolver_definitions.len() - 1).into());
                }
            }
        }
    }
    ctx.graph.resolver_definitions = resolver_definitions;

    Ok(())
}
