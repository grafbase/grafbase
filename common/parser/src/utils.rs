use async_graphql_parser::types::{BaseType, FieldDefinition};

/// Check if the given type is a primitive
///
/// A Primitive type is a custom scalar which can be cast into a primitive like an i32.
///   - Int
///   - Float
///   - String
///   - Boolean
///   - ID
pub(crate) fn is_type_primitive(field: &FieldDefinition) -> bool {
    match &field.ty.node.base {
        BaseType::Named(name) => match name.as_ref() {
            "String" => true,
            "Float" => true,
            "Boolean" => true,
            "ID" => true,
            "Int" => true,
            _ => false,
        },
        _ => false,
    }
}

/// Check if the given type is a non-nullable ID type
pub(crate) fn is_id_type_and_non_nullable(field: &FieldDefinition) -> bool {
    match &field.ty.node.base {
        BaseType::Named(name) => match name.as_ref() {
            "ID" => true,
            _ => false,
        },
        _ => false,
    }
}
