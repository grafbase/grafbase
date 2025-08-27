use super::*;
use crate::diagnostics::CompositeSchemasSourceSchemaValidationErrorCode;

/// https://graphql.github.io/composite-schemas-spec/draft/#sec-Query-Root-Type-Inaccessible
pub(crate) fn query_root_type_inaccessible(ctx: &mut ValidateContext<'_>) {
    for subgraph in ctx.subgraphs.iter_subgraph_views() {
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
pub(crate) fn lookup_returns_non_nullable_type(ctx: &mut ValidateContext<'_>, field: subgraphs::FieldWalker<'_>) {
    if field.r#type().is_required()
        && field
            .id
            .1
            .directives
            .iter_ir_directives(ctx.subgraphs)
            .any(|directive| matches!(directive, crate::composition_ir::Directive::CompositeLookup(_)))
    {
        let source_schema_name = field.parent_definition().subgraph().name().as_str();
        let parent_definition_name = field.parent_definition().name().as_str();
        let field_name = field.name().as_str();

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

pub(crate) fn override_from_self(ctx: &mut ValidateContext<'_>, field: subgraphs::FieldWalker<'_>) {
    let Some(r#override) = field.id.1.directives.r#override(ctx.subgraphs) else {
        return;
    };

    if r#override.from != field.parent_definition().subgraph().name().id {
        return;
    }

    ctx.diagnostics.push_composite_schemas_source_schema_validation_error(
        field.parent_definition().subgraph().name().as_str(),
        format!(
            r#"Source and destination subgraphs "{subgraph_name}" are the same for overridden field "{parent_definition_name}.{field_name}""#,
            subgraph_name = field.parent_definition().subgraph().name().as_str(),
            parent_definition_name = field.parent_definition().name().as_str(),
            field_name = field.name().as_str()
        ),
        CompositeSchemasSourceSchemaValidationErrorCode::OverrideFromSelf,
    );
}
