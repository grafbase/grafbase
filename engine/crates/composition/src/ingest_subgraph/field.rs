use super::*;
use crate::subgraphs::FieldId;

pub(crate) fn ingest_field_arguments(
    field_id: FieldId,
    arguments: &[Positioned<ast::InputValueDefinition>],
    matcher: &FederationDirectivesMatcher<'_>,
    subgraphs: &mut Subgraphs,
) {
    for argument in arguments {
        let r#type = &argument.node.ty.node;
        let type_id = subgraphs.intern_field_type(r#type);
        let arg_name = &argument.node.name.node;
        let is_inaccessible = has_inaccessible_directive(&argument.node.directives, matcher);

        subgraphs.push_field_argument(field_id, arg_name, type_id, is_inaccessible);
    }
}
