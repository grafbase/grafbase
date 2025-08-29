use super::*;

/// Group fields of the definitions that share the same name. For each name group, `compose_fn` is
/// called once with the relevant fields.
pub(super) fn for_each_field_group<'a>(
    subgraphs: &'a subgraphs::Subgraphs,
    definitions: &[DefinitionView<'a>],
    mut compose_fn: impl FnMut(&[subgraphs::FieldView<'a>]),
) {
    let mut all_fields = definitions
        .iter()
        .flat_map(|def| def.id.fields(subgraphs))
        .collect::<Vec<_>>();

    all_fields.sort_by_key(|field| field.name);

    let mut start = 0;
    let mut fields_buf = Vec::new();

    while start < all_fields.len() {
        fields_buf.clear();
        let field_name = all_fields[start].name;
        let end = all_fields[start..].partition_point(|field| field.name == field_name) + start;
        let fields = &all_fields[start..end];

        fields_buf.extend(fields.iter());

        compose_fn(&fields_buf);

        start = end;
    }
}

pub(super) fn compose_input_field_types<'a>(
    ctx: &mut Context<'a>,
    fields: impl Iterator<Item = subgraphs::FieldView<'a>>,
) -> Option<subgraphs::FieldType> {
    compose_field_types(ctx, fields, |a, b| a.compose_for_input(b))
}

pub(super) fn compose_output_field_types<'a>(
    ctx: &mut Context<'_>,
    fields: impl Iterator<Item = subgraphs::FieldView<'a>>,
) -> Option<subgraphs::FieldType> {
    compose_field_types(ctx, fields, |a, b| a.compose_for_output(b))
}

fn compose_field_types<'a>(
    ctx: &mut Context<'_>,
    fields: impl Iterator<Item = subgraphs::FieldView<'a>>,
    compose_fn: impl Fn(&subgraphs::FieldType, &subgraphs::FieldType) -> Option<subgraphs::FieldType>,
) -> Option<subgraphs::FieldType> {
    let mut fields = fields.map(|field| {
        let is_guest_batched = field.arguments(ctx.subgraphs).any(|arg| {
            arg.directives.iter_ir_directives(ctx.subgraphs).any(|dir| {
                let ir::Directive::CompositeRequire { field, .. } = dir else {
                    return false;
                };
                ctx[*field].trim_start().starts_with('[')
            })
        });
        let mut ty = field.r#type;
        if is_guest_batched {
            ty.wrapping = ty.wrapping.without_list().ok_or_else(|| {
                let parent_definition = ctx.subgraphs.at(field.parent_definition_id);
                format!(
                    "The field {}.{} has an argument with a batched @require, it must return a list",
                    ctx.subgraphs[parent_definition.name], ctx.subgraphs[field.name]
                )
            })?
        }
        Ok((field, ty))
    });

    let first = match fields.next()? {
        Ok(first) => first,
        Err(err) => {
            ctx.diagnostics.push_fatal(err);
            return None;
        }
    };

    match fields.try_fold(first, |(a_field, a_type), result| {
        let (b_field, b_type) = result?;
        compose_fn(&a_type, &b_type).map(|ty| (a_field, ty)).ok_or_else(|| {
            let parent_definition = ctx.subgraphs.at(first.0.parent_definition_id);
            let [a_field_subgraph, b_field_subgraph] = [a_field, b_field].map(|field| {
                ctx.subgraphs
                    .at(ctx.subgraphs.at(field.parent_definition_id).subgraph_id)
            });

            format!(
                "The {}.{} field has conflicting types in different subgraphs: {} in {} but {} in {}",
                ctx.subgraphs[parent_definition.name],
                ctx.subgraphs[first.0.name],
                a_field
                    .r#type
                    .wrapping
                    .type_display(&ctx.subgraphs[a_field.r#type.definition_name_id]),
                ctx.subgraphs[a_field_subgraph.name],
                b_field
                    .r#type
                    .wrapping
                    .type_display(&ctx.subgraphs[b_field.r#type.definition_name_id]),
                ctx.subgraphs[b_field_subgraph.name],
            )
        })
    }) {
        Ok((_, ty)) => Some(ty),
        Err(msg) => {
            ctx.diagnostics.push_fatal(msg);
            None
        }
    }
}

pub(super) fn compose_argument_types<'a>(
    parent_definition_name: StringId,
    mut arguments: impl Iterator<Item = &'a subgraphs::ArgumentRecord>,
    ctx: &mut Context<'a>,
) -> Option<subgraphs::FieldType> {
    let first = arguments.next()?;

    match arguments
        .map(|a| (a, &a.r#type))
        .try_fold((first, first.r#type), |(a_arg, a_type), (b_arg, b_type)| {
            a_type
                .compose_for_input(b_type)
                .map(|ty| (a_arg, ty))
                .ok_or((a_arg, b_arg))
        }) {
        Ok((_, ty)) => Some(ty),
        Err((a_arg, b_arg)) => {
            let [a_definition, b_definition] =
                [a_arg.parent_definition_id, b_arg.parent_definition_id].map(|id| ctx.subgraphs.at(id));
            let [a_subgraph, b_subgraph] = [a_definition, b_definition].map(|def| ctx.subgraphs.at(def.subgraph_id));

            ctx.diagnostics.push_fatal(format!(
                "The {}.{}({}:) argument has conflicting types in different subgraphs: {} in {} but {} in {}",
                ctx.subgraphs[parent_definition_name],
                ctx.subgraphs[a_arg.parent_field.1],
                ctx.subgraphs[a_arg.name],
                a_arg.r#type.display(ctx.subgraphs),
                ctx.subgraphs[a_subgraph.name],
                b_arg.r#type.display(ctx.subgraphs),
                ctx.subgraphs[b_subgraph.name],
            ));
            None
        }
    }
}
