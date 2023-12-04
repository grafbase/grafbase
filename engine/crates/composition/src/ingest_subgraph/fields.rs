use super::*;
use crate::subgraphs::FieldId;

pub(super) fn ingest_input_fields(
    parent_definition_id: DefinitionId,
    fields: &[Positioned<ast::InputValueDefinition>],
    matcher: &FederationDirectivesMatcher<'_>,
    subgraphs: &mut Subgraphs,
) {
    for field in fields {
        let field_type = subgraphs.intern_field_type(&field.node.ty.node);
        let directives = subgraphs.new_directive_container();

        directives::ingest_directives(directives, &field.node.directives, subgraphs, matcher);

        let description = field
            .node
            .description
            .as_ref()
            .map(|description| subgraphs.strings.intern(description.node.as_str()));

        subgraphs.push_field(subgraphs::FieldIngest {
            parent_definition_id,
            field_name: &field.node.name.node,
            field_type,
            directives,
            description,
        });
    }
}

fn ingest_field_arguments(
    field_id: FieldId,
    arguments: &[Positioned<ast::InputValueDefinition>],
    matcher: &FederationDirectivesMatcher<'_>,
    subgraphs: &mut Subgraphs,
) {
    for argument in arguments {
        let r#type = subgraphs.intern_field_type(&argument.node.ty.node);
        let name = subgraphs.strings.intern(argument.node.name.node.as_str());

        let argument_directives = subgraphs.new_directive_container();

        super::directives::ingest_directives(argument_directives, &argument.node.directives, subgraphs, matcher);

        let description = argument
            .node
            .description
            .as_ref()
            .map(|description| subgraphs.strings.intern(description.node.as_str()));

        subgraphs.insert_field_argument(field_id, name, r#type, argument_directives, description);
    }
}

pub(super) fn ingest_fields(
    definition_id: DefinitionId,
    fields: &[Positioned<ast::FieldDefinition>],
    federation_directives_matcher: &FederationDirectivesMatcher<'_>,
    subgraphs: &mut Subgraphs,
) {
    for field in fields {
        let field = &field.node;

        let description = field
            .description
            .as_ref()
            .map(|description| subgraphs.strings.intern(description.node.as_str()));

        let field_type = subgraphs.intern_field_type(&field.ty.node);
        let directives = subgraphs.new_directive_container();
        directives::ingest_directives(directives, &field.directives, subgraphs, federation_directives_matcher);

        let field_id = subgraphs.push_field(crate::subgraphs::FieldIngest {
            parent_definition_id: definition_id,
            field_name: &field.name.node,
            field_type,
            description,
            directives,
        });

        ingest_field_arguments(field_id, &field.arguments, federation_directives_matcher, subgraphs);
    }
}
