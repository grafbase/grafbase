use std::{borrow::Cow, collections::HashMap};

use case::CaseExt;
use grafbase_engine::{Name, Positioned};
use grafbase_engine_parser::types::{BaseType, FieldDefinition, Type, TypeDefinition, TypeKind};

// TODO: maybe get rid of this
fn is_str_type_primitive(name: &str) -> bool {
    matches!(name, "String" | "Float" | "Boolean" | "ID" | "Int")
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
        BaseType::Named(name) => is_str_type_primitive(name.as_ref()),
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

pub fn is_type_with_length(ty: &Type) -> bool {
    match &ty.base {
        BaseType::Named(name) => name.as_str() == "String",
        BaseType::List(_) => true,
    }
}

/// Check if the given type is a basic type
///
/// A BasicType is an Object and not an entity: it's not modelized.
#[allow(dead_code)]
pub fn is_type_basic_type(ctx: &HashMap<String, &'_ Positioned<TypeDefinition>>, ty: &Type) -> bool {
    let ty = get_base_from_type(ty);

    let is_a_basic_type = ctx
        .get(ty)
        .map(|type_def| !type_def.node.directives.iter().any(|directive| directive.is_model()))
        .expect("weird");

    is_a_basic_type
}

fn to_input_base_type(ctx: &HashMap<String, Cow<'_, Positioned<TypeDefinition>>>, base_type: BaseType) -> BaseType {
    match base_type {
        BaseType::Named(name) => {
            let ty = ctx.get(name.as_ref());
            let type_def = ty.map(|x| &x.node.kind);
            let is_modelized = ty
                .map(|ty| ty.node.directives.iter().any(|directive| directive.is_search()))
                .unwrap_or(false);
            match (type_def, is_modelized) {
                (Some(TypeKind::Scalar | TypeKind::Enum(_)), _) => BaseType::Named(name),
                (Some(TypeKind::Object(_)), false) => BaseType::Named(Name::new(format!("{name}Input"))),
                (Some(TypeKind::Object(_)), true) => BaseType::Named(Name::new("ID")),
                _ => BaseType::Named(Name::new("Error")),
            }
        }
        BaseType::List(list) => BaseType::List(Box::new(Type {
            base: to_input_base_type(ctx, list.base),
            nullable: list.nullable,
        })),
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
/// For a modelized type, the return type id `ID`.
///
/// # Examples
///
/// For String -> String
/// For Author -> AuthorInput
/// For [String!]! -> [String!]!
/// For [Author!] -> [AuthorInput!]
///
pub fn to_input_type(
    ctx: &HashMap<String, Cow<'_, Positioned<TypeDefinition>>>,
    Type { base, nullable }: Type,
) -> Type {
    Type {
        base: to_input_base_type(ctx, base),
        nullable,
    }
}

pub fn to_lower_camelcase<S: AsRef<str>>(field: S) -> String {
    field.as_ref().to_snake().to_camel_lowercase()
}
