use ast::InputValueDefinition;

use super::*;

pub(super) fn ingest_input_fields(
    ctx: &mut Context<'_>,
    parent_definition_id: DefinitionId,
    fields: ast::Iter<'_, ast::InputValueDefinition<'_>>,
) {
    for field in fields {
        let field_type = ctx.subgraphs.intern_field_type(field.ty());
        let directives = ctx.subgraphs.new_directive_site();
        let field_name = field.name();

        directives::ingest_directives(ctx, directives, field.directives(), |subgraphs| {
            let parent_definition = subgraphs.at(parent_definition_id);
            format!("{}.{field_name}", subgraphs[parent_definition.name])
        });

        let description = field
            .description()
            .map(|description| ctx.subgraphs.strings.intern(description.to_cow()));

        let default = field
            .default_value()
            .map(|default| crate::ast_value_to_subgraph_value(default, ctx.subgraphs));

        let name = ctx.subgraphs.strings.intern(field_name);

        ctx.subgraphs.push_field(subgraphs::FieldTuple {
            parent_definition_id,
            name,
            r#type: field_type,
            directives,
            description,
            input_field_default_value: default,
        });
    }
}

fn ingest_field_arguments(
    ctx: &mut Context<'_>,
    parent_definition_id: DefinitionId,
    parent_field_name: subgraphs::StringId,
    arguments: ast::iter::Iter<'_, InputValueDefinition<'_>>,
) {
    for argument in arguments {
        let r#type = ctx.subgraphs.intern_field_type(argument.ty());
        let name = ctx.subgraphs.strings.intern(argument.name());

        let directives = ctx.subgraphs.new_directive_site();

        ingest_directives(ctx, directives, argument.directives(), |subgraphs| {
            let field_name = &subgraphs[parent_field_name];
            let parent_definition_name = &subgraphs[subgraphs.at(parent_definition_id).name];
            format!("{}.{}({}:)", parent_definition_name, field_name, argument.name())
        });

        let description = argument
            .description()
            .as_ref()
            .map(|description| ctx.subgraphs.strings.intern(description.to_cow()));

        let default_value = argument
            .default_value()
            .map(|default| ast_value_to_subgraph_value(default, ctx.subgraphs));

        ctx.subgraphs.insert_field_argument(subgraphs::ArgumentRecord {
            parent_definition_id,
            parent_field_name,
            name,
            r#type,
            directives,
            description,
            default_value,
        });
    }
}

pub(super) fn ingest_fields(
    ctx: &mut Context<'_>,
    definition_id: DefinitionId,
    fields: ast::iter::Iter<'_, ast::FieldDefinition<'_>>,
    parent_is_query_root_type: bool,
) {
    for field in fields {
        let field_name = field.name();

        // These are special fields on Query exposed by subgraphs.
        if parent_is_query_root_type && ["_entities", "_service"].contains(&field_name) {
            continue;
        }

        let description = field
            .description()
            .map(|description| ctx.subgraphs.strings.intern(description.to_cow()));

        let field_type = ctx.subgraphs.intern_field_type(field.ty());
        let directives = ctx.subgraphs.new_directive_site();
        let field_name_id = ctx.subgraphs.strings.intern(field_name);

        ctx.subgraphs.push_field(crate::subgraphs::FieldTuple {
            parent_definition_id: definition_id,
            name: field_name_id,
            r#type: field_type,
            description,
            directives,
            input_field_default_value: None,
        });

        directives::ingest_directives(ctx, directives, field.directives(), |subgraphs| {
            let definition = subgraphs.at(definition_id);
            format!("{}.{}", subgraphs[definition.name], field_name)
        });

        ingest_field_arguments(ctx, definition_id, field_name_id, field.arguments());
    }
}
