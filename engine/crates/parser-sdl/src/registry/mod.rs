//! TODO:
//!
//! -> Split each of the creation and add tests with SDL
//!

use case::CaseExt;
use engine::{registry::MetaInputValue, validation::dynamic_validators::DynValidator};
use engine_parser::types::{FieldDefinition, ObjectType};

use crate::{
    rules::{
        length_directive::{LENGTH_DIRECTIVE, MAX_ARGUMENT, MIN_ARGUMENT},
        visitor::VisitorContext,
    },
    utils::{to_base_type_str, to_input_type},
};

pub mod names;
pub(crate) mod pagination;

/// Create an input type for a non_primitive Type.
pub fn add_input_type_non_primitive(ctx: &mut VisitorContext<'_>, object: &ObjectType, type_name: &str) -> String {
    let type_name = type_name.to_string();
    let input_type = format!("{}Input", type_name.to_camel());
    let fields = object
        .fields
        .iter()
        .filter_map(|field| {
            let field_ty = to_base_type_str(&field.node.ty.node.base);
            match ctx.types.get(&field_ty) {
                Some(field_type_definition) => {
                    if field_type_definition
                        .directives
                        .iter()
                        .any(|directive| directive.is_model())
                    {
                        ctx.report_error(
                            vec![field.pos],
                            format!(
                                "Non @model type ({ty}) cannot have a field ({field}) with a @model type ({field_ty}). Consider adding @model directive to {ty}.",
                                ty = type_name,
                                field = field.node.name,
                            ),
                        );
                        None
                    } else {
                        Some(MetaInputValue {
                            description: field.node.description.clone().map(|x| x.node),
                            ..MetaInputValue::new(
                                field.name.node.to_string(),
                                to_input_type(&ctx.types, field.node.ty.clone().node).to_string(),
                            )
                            .with_rename(field.mapped_name().map(ToString::to_string))
                        })
                    }
                }
                None => {
                    ctx.report_error(vec![field.pos], format!("Unknown type: {field_ty}"));
                    None
                }
            }
        })
        .collect::<Vec<_>>();

    // Input
    ctx.registry.get_mut().create_type(
        |_| {
            engine::registry::InputObjectType::new(input_type.clone(), fields)
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
                    if let engine_value::ConstValue::Number(ref min) = argument.node {
                        min.as_u64().and_then(|min| min.try_into().ok())
                    } else {
                        None
                    }
                })
            });
            DynValidator::length(min_value, max_value)
        })
}
