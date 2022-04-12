use async_graphql_parser::types::{BaseType, FieldDefinition};

/// Check if the given type is a primitive
///
/// A Primitive type is a custom scalar which can be cast into a primitive like an i32.
///   - Int
///   - Float
///   - String
///   - Boolean
///   - ID
pub fn is_type_primitive(field: &FieldDefinition) -> bool {
    match &field.ty.node.base {
        BaseType::Named(name) => matches!(name.as_ref(), "String" | "Float" | "Boolean" | "ID" | "Int"),
        _ => false,
    }
}

/// Check if the given type is a non-nullable ID type
pub fn is_id_type_and_non_nullable(field: &FieldDefinition) -> bool {
    match &field.ty.node.base {
        BaseType::Named(name) => matches!(name.as_ref(), "ID"),
        _ => false,
    }
}
