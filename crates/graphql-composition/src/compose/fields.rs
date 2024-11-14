use super::*;

/// Group fields of the definitions that share the same name. For each name group, `compose_fn` is
/// called once with the relevant fields.
pub(super) fn for_each_field_group<'a>(
    definitions: &[DefinitionWalker<'a>],
    mut compose_fn: impl FnMut(&[FieldWalker<'a>]),
) {
    let mut all_fields = definitions
        .iter()
        .flat_map(|def| def.fields().map(|field| (field.name().id, field)))
        .collect::<Vec<_>>();

    all_fields.sort_by_key(|(name, _)| *name);

    let mut start = 0;
    let mut fields_buf = Vec::new();

    while start < all_fields.len() {
        fields_buf.clear();
        let field_name = all_fields[start].0;
        let end = all_fields[start..].partition_point(|(name, _)| *name == field_name) + start;
        let fields = &all_fields[start..end];

        fields_buf.extend(fields.iter().map(|(_, field)| field));

        compose_fn(&fields_buf);

        start = end;
    }
}

pub(super) fn compose_input_field_types<'a>(
    fields: impl Iterator<Item = FieldWalker<'a>>,
    ctx: &mut Context<'_>,
) -> Option<subgraphs::FieldTypeId> {
    compose_field_types(fields, ctx, |a, b| a.compose_for_input(b))
}

pub(super) fn compose_output_field_types<'a>(
    fields: impl Iterator<Item = FieldWalker<'a>>,
    ctx: &mut Context<'_>,
) -> Option<subgraphs::FieldTypeId> {
    compose_field_types(fields, ctx, |a, b| a.compose_for_output(b))
}

fn compose_field_types<'a>(
    mut fields: impl Iterator<Item = FieldWalker<'a>>,
    ctx: &mut Context<'_>,
    compose_fn: impl Fn(
        subgraphs::FieldTypeWalker<'a>,
        subgraphs::FieldTypeWalker<'a>,
    ) -> Option<subgraphs::FieldTypeWalker<'a>>,
) -> Option<subgraphs::FieldTypeId> {
    let first = fields.next()?;

    match fields
        .map(|f| (f, f.r#type()))
        .try_fold((first, first.r#type()), |(a_field, a_type), (b_field, b_type)| {
            compose_fn(a_type, b_type)
                .map(|ty| (a_field, ty))
                .ok_or((a_field, b_field))
        }) {
        Ok((_, ty)) => Some(ty.id),
        Err((a_field, b_field)) => {
            ctx.diagnostics.push_fatal(format!(
                "The {}.{} field has conflicting types in different subgraphs: {} in {} but {} in {}",
                first.parent_definition().name().as_str(),
                first.name().as_str(),
                a_field.r#type(),
                a_field.parent_definition().subgraph().name().as_str(),
                b_field.r#type(),
                b_field.parent_definition().subgraph().name().as_str(),
            ));
            None
        }
    }
}

pub(super) fn compose_argument_types<'a>(
    parent_definition_name: StringId,
    field_name: StringId,
    mut arguments: impl Iterator<Item = subgraphs::FieldArgumentWalker<'a>>,
    ctx: &mut Context<'a>,
) -> Option<subgraphs::FieldTypeId> {
    let first = arguments.next()?;

    match arguments
        .map(|a| (a, a.r#type()))
        .try_fold((first, first.r#type()), |(a_arg, a_type), (b_arg, b_type)| {
            a_type
                .compose_for_input(b_type)
                .map(|ty| (a_arg, ty))
                .ok_or((a_arg, b_arg))
        }) {
        Ok((_, ty)) => Some(ty.id),
        Err((a_arg, b_arg)) => {
            ctx.diagnostics.push_fatal(format!(
                "The {}.{}({}:) argument has conflicting types in different subgraphs: {} in {} but {} in {}",
                ctx.subgraphs.walk(parent_definition_name).as_str(),
                ctx.subgraphs.walk(field_name).as_str(),
                a_arg.name().as_str(),
                a_arg.r#type(),
                a_arg.field().parent_definition().subgraph().name().as_str(),
                b_arg.r#type(),
                b_arg.field().parent_definition().subgraph().name().as_str(),
            ));
            None
        }
    }
}

pub(super) fn insert_list_size_directives<'a>(
    fields: impl Iterator<Item = &'a subgraphs::Walker<'a, (subgraphs::FieldId, subgraphs::FieldTuple)>>,
    ctx: &mut ComposeContext<'_>,
    definition_name: federated::StringId,
    field_name: federated::StringId,
) {
    let list_size_directive = fields
        .filter_map(|field| field.directives().list_size().cloned())
        .reduce(|lhs, rhs| lhs.merge(rhs));

    if let Some(directive) = list_size_directive {
        ctx.insert_list_size_directive(definition_name, field_name, directive);
    }
}
