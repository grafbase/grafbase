use async_graphql_parser::Positioned;

use super::*;
use crate::subgraphs::FieldId;

pub(crate) fn ingest_field_arguments(
    field_id: FieldId,
    arguments: &[Positioned<ast::InputValueDefinition>],
    subgraphs: &mut Subgraphs,
) {
    for argument in arguments {
        let r#type = &argument.node.ty.node;
        let type_id = subgraphs.intern_field_type(r#type);
        let arg_name = &argument.node.name.node;
        subgraphs.push_field_argument(field_id, arg_name, type_id);
    }
}
