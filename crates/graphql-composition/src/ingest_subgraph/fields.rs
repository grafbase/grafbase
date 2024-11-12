use ast::InputValueDefinition;

use super::*;
use crate::subgraphs::FieldId;

pub(super) fn ingest_input_fields(
    parent_definition_id: DefinitionId,
    fields: ast::Iter<'_, ast::InputValueDefinition<'_>>,
    matcher: &DirectiveMatcher<'_>,
    subgraphs: &mut Subgraphs,
    subgraph_id: SubgraphId,
) {
    for field in fields {
        let field_type = subgraphs.intern_field_type(field.ty());
        let directives = subgraphs.new_directive_site();
        let field_name = field.name();

        directives::ingest_directives(
            directives,
            field.directives(),
            subgraphs,
            matcher,
            subgraph_id,
            |subgraphs| format!("{}.{field_name}", subgraphs.walk(parent_definition_id).name().as_str(),),
        );

        let description = field
            .description()
            .map(|description| subgraphs.strings.intern(description.to_cow()));

        let default = field
            .default_value()
            .map(|default| crate::ast_value_to_subgraph_value(default, subgraphs));

        subgraphs.push_field(subgraphs::FieldIngest {
            parent_definition_id,
            field_name,
            field_type,
            directives,
            description,
            default,
        });
    }
}

fn ingest_field_arguments(
    field_id: FieldId,
    arguments: ast::iter::Iter<'_, InputValueDefinition<'_>>,
    matcher: &DirectiveMatcher<'_>,
    subgraph_id: SubgraphId,
    subgraphs: &mut Subgraphs,
) {
    for argument in arguments {
        let r#type = subgraphs.intern_field_type(argument.ty());
        let name = subgraphs.strings.intern(argument.name());

        let argument_directives = subgraphs.new_directive_site();

        ingest_directives(
            argument_directives,
            argument.directives(),
            subgraphs,
            matcher,
            subgraph_id,
            |subgraphs| {
                let field = subgraphs.walk_field(field_id);
                format!(
                    "{}.{}({}:)",
                    field.parent_definition().name().as_str(),
                    field.name().as_str(),
                    argument.name()
                )
            },
        );

        let description = argument
            .description()
            .as_ref()
            .map(|description| subgraphs.strings.intern(description.to_cow()));

        let default = argument
            .default_value()
            .map(|default| ast_value_to_subgraph_value(default, subgraphs));

        subgraphs.insert_field_argument(field_id, name, r#type, argument_directives, description, default);
    }
}

pub(super) fn ingest_fields(
    definition_id: DefinitionId,
    fields: ast::iter::Iter<'_, ast::FieldDefinition<'_>>,
    directive_matcher: &DirectiveMatcher<'_>,
    parent_is_query_root_type: bool,
    subgraph_id: SubgraphId,
    subgraphs: &mut Subgraphs,
) {
    for field in fields {
        let field_name = field.name();

        // These are special fields on Query exposed by subgraphs.
        if parent_is_query_root_type && ["_entities", "_service"].contains(&field_name) {
            continue;
        }

        let description = field
            .description()
            .map(|description| subgraphs.strings.intern(description.to_cow()));

        let field_type = subgraphs.intern_field_type(field.ty());
        let directives = subgraphs.new_directive_site();

        let field_id = subgraphs.push_field(crate::subgraphs::FieldIngest {
            parent_definition_id: definition_id,
            field_name,
            field_type,
            description,
            directives,
            default: None,
        });

        directives::ingest_directives(
            directives,
            field.directives(),
            subgraphs,
            directive_matcher,
            subgraph_id,
            |subgraphs| format!("{}.{}", subgraphs.walk(definition_id).name().as_str(), field_name),
        );

        ingest_field_arguments(field_id, field.arguments(), directive_matcher, subgraph_id, subgraphs);
    }
}
