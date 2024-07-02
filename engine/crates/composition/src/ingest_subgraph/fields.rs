use super::*;
use crate::subgraphs::FieldId;

pub(super) fn ingest_input_fields(
    parent_definition_id: DefinitionId,
    fields: &[Positioned<ast::InputValueDefinition>],
    matcher: &DirectiveMatcher<'_>,
    subgraphs: &mut Subgraphs,
    subgraph_id: SubgraphId,
) {
    for field in fields {
        let field_type = subgraphs.intern_field_type(&field.node.ty.node);
        let directives = subgraphs.new_directive_site();
        let field_name = field.node.name.node.as_str();

        directives::ingest_directives(
            directives,
            &field.node.directives,
            subgraphs,
            matcher,
            subgraph_id,
            |subgraphs| {
                format!(
                    "{}.{}",
                    subgraphs.walk(parent_definition_id).name().as_str(),
                    field.node.name.node.as_str()
                )
            },
        );

        let description = field
            .node
            .description
            .as_ref()
            .map(|description| subgraphs.strings.intern(description.node.as_str()));

        let default = field
            .node
            .default_value
            .as_ref()
            .map(|default| crate::ast_value_to_subgraph_value(&default.node, subgraphs));

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
    arguments: &[Positioned<ast::InputValueDefinition>],
    matcher: &DirectiveMatcher<'_>,
    subgraph_id: SubgraphId,
    subgraphs: &mut Subgraphs,
) {
    for argument in arguments {
        let r#type = subgraphs.intern_field_type(&argument.node.ty.node);
        let name = subgraphs.strings.intern(argument.node.name.node.as_str());

        let argument_directives = subgraphs.new_directive_site();

        ingest_directives(
            argument_directives,
            &argument.node.directives,
            subgraphs,
            matcher,
            subgraph_id,
            |subgraphs| {
                let field = subgraphs.walk_field(field_id);
                format!(
                    "{}.{}({}:)",
                    field.parent_definition().name().as_str(),
                    field.name().as_str(),
                    argument.node.name.node
                )
            },
        );

        let description = argument
            .node
            .description
            .as_ref()
            .map(|description| subgraphs.strings.intern(description.node.as_str()));

        let default = argument
            .node
            .default_value
            .as_ref()
            .map(|default| ast_value_to_subgraph_value(&default.node, subgraphs));

        subgraphs.insert_field_argument(field_id, name, r#type, argument_directives, description, default);
    }
}

pub(super) fn ingest_fields(
    definition_id: DefinitionId,
    fields: &[Positioned<ast::FieldDefinition>],
    directive_matcher: &DirectiveMatcher<'_>,
    parent_is_query_root_type: bool,
    subgraph_id: SubgraphId,
    subgraphs: &mut Subgraphs,
) {
    for field in fields {
        let field = &field.node;
        let field_name = field.name.node.as_str();

        // These are special fields on Query exposed by subgraphs.
        if parent_is_query_root_type && ["_entities", "_service"].contains(&field_name) {
            continue;
        }

        let description = field
            .description
            .as_ref()
            .map(|description| subgraphs.strings.intern(description.node.as_str()));

        let field_type = subgraphs.intern_field_type(&field.ty.node);
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
            &field.directives,
            subgraphs,
            directive_matcher,
            subgraph_id,
            |subgraphs| format!("{}.{}", subgraphs.walk(definition_id).name().as_str(), field_name),
        );

        ingest_field_arguments(field_id, &field.arguments, directive_matcher, subgraph_id, subgraphs);
    }
}
