use crate::rules::model_directive::MODEL_DIRECTIVE;
use async_graphql::{Name, Positioned};
use async_graphql_parser::types::{BaseType, FieldDefinition, Type, TypeDefinition};
use std::collections::HashMap;

fn is_type_primitive_internal(name: &Name) -> bool {
    matches!(name.as_ref(), "String" | "Float" | "Boolean" | "ID" | "Int")
}

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
        BaseType::Named(name) => is_type_primitive_internal(name),
        _ => false,
    }
}

#[allow(dead_code)]
fn get_base_from_type(ty: &Type) -> &str {
    match &ty.base {
        BaseType::Named(name) => name.as_str(),
        BaseType::List(ty_boxed) => get_base_from_type(ty_boxed.as_ref()),
    }
}

/// Check if the given type is a basic type
///
/// A BasicType is an Object and not an entity: it's not modelized.
#[allow(dead_code)]
pub fn is_type_basic_type<'a>(ctx: &HashMap<String, &'a Positioned<TypeDefinition>>, ty: &Type) -> bool {
    let ty = get_base_from_type(ty);

    let is_a_basic_type = ctx
        .get(ty)
        .map(|type_def| {
            !type_def
                .node
                .directives
                .iter()
                .any(|directive| directive.node.name.node == MODEL_DIRECTIVE)
        })
        .expect("weird");

    is_a_basic_type
}

fn to_input_base_type(base_type: BaseType) -> BaseType {
    match base_type {
        BaseType::Named(name) => {
            if is_type_primitive_internal(&name) {
                BaseType::Named(name)
            } else {
                BaseType::Named(Name::new(format!("{}Input", name)))
            }
        }
        BaseType::List(list) => to_input_base_type(list.base),
    }
}

/// Get the base type string for a type.
pub fn to_base_type_str(ty: &BaseType) -> String {
    match ty {
        BaseType::Named(name) => name.to_string(),
        BaseType::List(ty_list) => to_base_type_str(&ty_list.base),
    }
}

/// Transform a type into his associated input Type.
/// The type must not be a modelized type.
///
/// For String -> String
/// For Author -> AuthorInput
/// For [String!]! -> [String!]!
/// For [Author!] -> [AuthorInput!]
pub fn to_input_type(Type { base, nullable }: Type) -> Type {
    Type {
        base: to_input_base_type(base),
        nullable,
    }
}

/// Check if the given type is a non-nullable ID type
pub fn is_id_type_and_non_nullable(field: &FieldDefinition) -> bool {
    match &field.ty.node.base {
        BaseType::Named(name) => matches!(name.as_ref(), "ID"),
        _ => false,
    }
}
