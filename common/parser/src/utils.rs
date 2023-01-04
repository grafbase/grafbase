use crate::rules::model_directive::MODEL_DIRECTIVE;
use case::CaseExt;
use dynaql::{indexmap::IndexMap, registry::MetaInputValue};
use dynaql::{Name, Positioned};
use dynaql_parser::types::{BaseType, FieldDefinition, Type, TypeDefinition, TypeKind};
use std::borrow::Cow;
use std::collections::HashMap;

fn is_type_primitive_internal(name: &str) -> bool {
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
        BaseType::Named(name) => is_type_primitive_internal(name.as_ref()),
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

fn to_input_base_type(ctx: &HashMap<String, Cow<'_, Positioned<TypeDefinition>>>, base_type: BaseType) -> BaseType {
    match base_type {
        BaseType::Named(name) => {
            let ty = ctx.get(name.as_ref());
            let type_def = ty.map(|x| &x.node.kind);
            let is_modelized = ty
                .map(|ty| {
                    ty.node
                        .directives
                        .iter()
                        .any(|directive| directive.node.name.node == MODEL_DIRECTIVE)
                })
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

pub fn pagination_arguments() -> IndexMap<String, MetaInputValue> {
    IndexMap::from([
        (
            "after".to_owned(),
            MetaInputValue {
                name: "after".to_owned(),
                description: None,
                ty: "String".to_string(),
                default_value: None,
                validators: None,
                visible: None,
                is_secret: false,
            },
        ),
        (
            "before".to_owned(),
            MetaInputValue {
                name: "before".to_owned(),
                description: None,
                ty: "String".to_string(),
                default_value: None,
                validators: None,
                visible: None,
                is_secret: false,
            },
        ),
        (
            "first".to_owned(),
            MetaInputValue {
                name: "first".to_owned(),
                description: None,
                ty: "Int".to_string(),
                default_value: None,
                validators: None,
                visible: None,
                is_secret: false,
            },
        ),
        (
            "last".to_owned(),
            MetaInputValue {
                name: "last".to_owned(),
                description: None,
                ty: "Int".to_string(),
                default_value: None,
                validators: None,
                visible: None,
                is_secret: false,
            },
        ),
    ])
}
