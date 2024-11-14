use graphql_federated_graph::directives::ListSizeDirective;

use super::{context::Context, federated, CompositionIr};

pub fn emit_list_sizes(ir: &CompositionIr, ctx: &mut Context<'_>) {
    for ((definition_name, field_name), list_size) in &ir.list_sizes {
        let Some(definition) = ctx.definitions.get(definition_name) else {
            continue;
        };
        let Some(field_id) = ctx.selection_map.get(&(*definition, *field_name)).copied() else {
            continue;
        };
        let field = &ctx.out[field_id];
        let ListSizeDirective {
            assumed_size,
            slicing_arguments,
            sized_fields,
            require_one_slicing_argument,
        } = list_size;

        let argument_base_id = field.arguments.0;
        let arguments = &ctx.out[field.arguments];
        let slicing_arguments = slicing_arguments
            .iter()
            .filter_map(|argument| {
                let (index, _) = arguments
                    .iter()
                    .enumerate()
                    .find(|(_, value)| ctx.lookup_string_id(value.name) == *argument)?;

                Some(federated::InputValueDefinitionId::from(
                    index + usize::from(argument_base_id),
                ))
            })
            .collect();

        let child_type_id = field.r#type.definition;
        let sized_fields = sized_fields
            .iter()
            .filter_map(|field| {
                let field_name = ctx.lookup_str(field)?;
                ctx.selection_map.get(&(child_type_id, field_name)).copied()
            })
            .collect();

        ctx.out.list_sizes.push((
            field_id,
            federated::ListSize {
                assumed_size: *assumed_size,
                slicing_arguments,
                sized_fields,
                require_one_slicing_argument: *require_one_slicing_argument,
            },
        ));
    }
    ctx.out.list_sizes.sort_by_key(|(field_id, _)| *field_id);
}
