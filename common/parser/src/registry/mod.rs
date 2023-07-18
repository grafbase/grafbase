//! TODO:
//!
//! -> Split each of the creation and add tests with SDL
//!
use case::CaseExt;

use dynaql::registry::enums::DynaqlEnum;
use dynaql::registry::relations::MetaRelation;
use dynaql::registry::{self, MetaInputValue, NamedType};
use dynaql::registry::{MetaEnumValue, Registry};
use dynaql::validation::dynamic_validators::DynValidator;
use dynaql_parser::types::{FieldDefinition, ObjectType, TypeDefinition};

use std::fmt::Display;

use crate::registry::names::MetaNames;
use crate::rules::length_directive::{LENGTH_DIRECTIVE, MAX_ARGUMENT, MIN_ARGUMENT};
use crate::rules::visitor::VisitorContext;
use crate::utils::to_input_type;

mod create_update;
mod delete;
pub mod names;
pub(crate) mod pagination;
mod relations;
mod search;
pub use create_update::{add_mutation_create, add_mutation_update, NumericFieldKind};
pub use delete::add_mutation_delete;
pub use pagination::{add_query_paginated_collection, generate_pagination_args};
pub use search::add_query_search;

pub fn register_dynaql_enum<T: DynaqlEnum>(registry: &mut Registry) -> NamedType<'static> {
    let type_name = T::ty().to_string();
    registry.create_type(
        |_| registry::EnumType::new(type_name.clone(), T::values().into_iter().map(MetaEnumValue::new)).into(),
        &type_name,
        &type_name,
    );
    type_name.into()
}

/// Create an input type for a non_primitive Type.
pub fn add_input_type_non_primitive(ctx: &mut VisitorContext<'_>, object: &ObjectType, type_name: &str) -> String {
    let type_name = type_name.to_string();
    let input_type = format!("{}Input", type_name.to_camel());

    // Input
    ctx.registry.get_mut().create_type(
        |_| {
            dynaql::registry::InputObjectType::new(
                input_type.clone(),
                object.fields.iter().map(|field| MetaInputValue {
                    description: field.node.description.clone().map(|x| x.node),
                    ..MetaInputValue::new(
                        field.name.node.to_string(),
                        to_input_type(&ctx.types, field.node.ty.clone().node).to_string(),
                    )
                    .with_rename(field.mapped_name().map(ToString::to_string))
                }),
            )
            .with_description(Some(format!("{type_name} input type.")))
            .into()
        },
        &input_type,
        &input_type,
    );

    input_type
}

pub fn get_length_validator(field: &FieldDefinition) -> Option<DynValidator> {
    use tuple::Map;
    field
        .directives
        .iter()
        .find(|directive| directive.node.name.node == LENGTH_DIRECTIVE)
        .map(|directive| {
            let (min_value, max_value) = (MIN_ARGUMENT, MAX_ARGUMENT).map(|argument_name| {
                directive.node.get_argument(argument_name).and_then(|argument| {
                    if let dynaql_value::ConstValue::Number(ref min) = argument.node {
                        min.as_u64().and_then(|min| min.try_into().ok())
                    } else {
                        None
                    }
                })
            });
            DynValidator::length(min_value, max_value)
        })
}

/// Used to keep track of the parent relation when created nested input types
/// TODO: Merge it with MetaRelation?
pub struct ParentRelation<'a> {
    /// TypeDefinition of @model type
    model_type_definition: &'a TypeDefinition,
    meta: &'a MetaRelation,
}

impl<'a> Display for ParentRelation<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} relation of {}",
            self.meta.name,
            MetaNames::model(self.model_type_definition)
        )
    }
}
