use super::*;
use crate::diagnostics::CompositeSchemasSourceSchemaValidationErrorCode;

/// https://graphql.github.io/composite-schemas-spec/draft/#sec-Query-Root-Type-Inaccessible
pub(crate) fn query_root_type_inaccessible(ctx: &mut ValidateContext<'_>) {
    for subgraph in ctx.subgraphs.iter_subgraphs() {
        let Some(query_root) = subgraph.query_type else {
            continue;
        };

        let directives = ctx.subgraphs.at(query_root).directives;

        if !directives.inaccessible(ctx.subgraphs) {
            continue;
        }

        let subgraph_name = &ctx.subgraphs[subgraph.name];
        ctx.diagnostics.push_composite_schemas_source_schema_validation_error(
            subgraph_name,
            format_args!("The query root type cannot be inaccessible"),
            CompositeSchemasSourceSchemaValidationErrorCode::QueryRootTypeInaccessible,
        );
    }
}

/// https://graphql.github.io/composite-schemas-spec/draft/#sec-Lookup-Returns-Non-Nullable-Type
pub(crate) fn lookup_returns_non_nullable_type(ctx: &mut ValidateContext<'_>, field: subgraphs::FieldView<'_>) {
    if field.r#type.is_required()
        && field
            .directives
            .iter_ir_directives(ctx.subgraphs)
            .any(|directive| matches!(directive, crate::composition_ir::Directive::CompositeLookup(_)))
    {
        let parent_definition = ctx.subgraphs.at(field.parent_definition_id);
        let source_schema_name = ctx.subgraphs[ctx.subgraphs.at(parent_definition.subgraph_id).name].as_ref();
        let parent_definition_name = ctx.subgraphs[parent_definition.name].as_ref();
        let field_name = ctx.subgraphs[field.name].as_ref();

        let message = format!(
            "The \"{parent_definition_name}.{field_name}\" lookup field is required, but fields annotated with @lookup should be nullable.",
        );

        ctx.diagnostics.push_composite_schemas_source_schema_validation_error(
            source_schema_name,
            message,
            CompositeSchemasSourceSchemaValidationErrorCode::LookupReturnsNonNullableType,
        );
    }
}

pub(crate) fn override_from_self(ctx: &mut ValidateContext<'_>, field: subgraphs::FieldView<'_>) {
    let Some(r#override) = field.directives.r#override(ctx.subgraphs) else {
        return;
    };

    let parent_definition = ctx.subgraphs.at(field.parent_definition_id);
    let parent_subgraph = ctx.subgraphs.at(parent_definition.subgraph_id);

    if r#override.from != parent_subgraph.name {
        return;
    }

    ctx.diagnostics.push_composite_schemas_source_schema_validation_error(
        &ctx.subgraphs[parent_subgraph.name],
        format!(
            r#"Source and destination subgraphs "{subgraph_name}" are the same for overridden field "{parent_definition_name}.{field_name}""#,
            subgraph_name = ctx.subgraphs[parent_subgraph.name],
            parent_definition_name = ctx.subgraphs[parent_definition.name],
            field_name = ctx.subgraphs[field.name]
        ),
        CompositeSchemasSourceSchemaValidationErrorCode::OverrideFromSelf,
    );
}
