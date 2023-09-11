use engine::{
    names::{MONGODB_OUTPUT_FIELD_ID, OUTPUT_FIELD_ID},
    registry::{EnumType, InputObjectType, MetaEnumValue, MetaInputValue},
};
use engine_parser::types::{BaseType, ObjectType, Type};

use super::generic;
use crate::{
    registry::names::MetaNames,
    rules::{mongodb_directive::CreateTypeContext, visitor::VisitorContext},
};

const LOGICAL_OPERATIONS: &[(&str, &str, &str)] = &[
    ("ALL", "$and", "All of the filters must match"),
    ("NONE", "$nor", "None of the filters must match"),
    ("ANY", "$or", "At least one of the filters must match"),
];

pub(crate) fn register_input(visitor_ctx: &mut VisitorContext<'_>, create_ctx: &CreateTypeContext<'_>) -> String {
    let input_type_name = MetaNames::collection(create_ctx.r#type);

    let implicit_fields = std::iter::once({
        let mut input = MetaInputValue::new(OUTPUT_FIELD_ID, generic::filter_type_name("ID"));
        input.rename = Some(MONGODB_OUTPUT_FIELD_ID.to_string());

        input
    });

    let logical_array_fields = LOGICAL_OPERATIONS.iter().map(|(name, rename, description)| {
        let r#type = format!("[{input_type_name}]");

        let mut input = MetaInputValue::new(*name, r#type);
        input.description = Some((*description).to_string());
        input.rename = Some((*rename).to_string());

        input
    });

    register_type_input(
        visitor_ctx,
        create_ctx.object,
        &input_type_name,
        implicit_fields.chain(logical_array_fields),
    );

    input_type_name
}

pub(crate) fn register_type_input<'a>(
    visitor_ctx: &mut VisitorContext<'_>,
    object: &ObjectType,
    input_type_name: &str,
    extra_fields: impl Iterator<Item = MetaInputValue> + 'a,
) {
    let explicit_fields = object.fields.iter().map(|field| {
        let r#type = if field.r#type().base.is_list() {
            let base = field.r#type().base.to_base_type_str();
            generic::filter_type_name(&format!("{base}Array"))
        } else {
            let base = field.r#type().base.to_base_type_str();
            generic::filter_type_name(base)
        };

        let mut input = MetaInputValue::new(field.node.name.node.to_string(), r#type);

        input.description = field.description().map(ToString::to_string);
        input.rename = field.mapped_name().map(ToString::to_string);

        input
    });

    let input_fields = extra_fields.chain(explicit_fields);
    let input_type = InputObjectType::new(input_type_name.to_string(), input_fields);

    visitor_ctx
        .registry
        .borrow_mut()
        .create_type(|_| input_type.into(), input_type_name, input_type_name);
}

pub(crate) fn register_orderby_input<'a>(
    visitor_ctx: &mut VisitorContext<'_>,
    object: &ObjectType,
    type_name: &str,
    extra_fields: impl Iterator<Item = (&'a str, &'a str)> + 'a,
) -> String {
    let input_type_name = MetaNames::pagination_orderby_input_by_str(type_name).to_string();
    let direction_type = register_mongo_order_direction(visitor_ctx);

    let extra_fields = extra_fields.map(|(name, rename)| {
        let mut input = MetaInputValue::new(name.to_string(), Type::nullable(direction_type.clone()).to_string());
        input.rename = Some(rename.to_string());
        input
    });

    let input_fields = object.fields.iter().map(|field| {
        let registry = visitor_ctx.registry.borrow();
        let composite = registry.types.get(field.ty.base.to_base_type_str());

        let type_name = match composite {
            Some(composite) if composite.is_object() => {
                MetaNames::pagination_orderby_input_by_str(composite.name()).to_string()
            }
            _ => direction_type.to_string(),
        };

        let mut input = MetaInputValue::new(field.node.name.node.to_string(), type_name);
        input.rename = field.mapped_name().map(ToString::to_string);
        input
    });

    let fields = extra_fields.chain(input_fields);
    let input_object = InputObjectType::new(input_type_name.to_string(), fields).with_oneof(true);

    visitor_ctx
        .registry
        .borrow_mut()
        .create_type(|_| input_object.into(), &input_type_name, &input_type_name);

    input_type_name
}

fn register_mongo_order_direction(visitor_ctx: &mut VisitorContext<'_>) -> BaseType {
    let type_name = "MongoOrderByDirection";

    let variants = [("ASC", "1"), ("DESC", "-1")].iter().map(|(name, value)| {
        let mut variant = MetaEnumValue::new((*name).to_string());
        variant.value = Some((*value).to_string());

        variant
    });

    let r#enum = EnumType::new(type_name.to_string(), variants);

    visitor_ctx
        .registry
        .borrow_mut()
        .create_type(|_| r#enum.into(), type_name, type_name);

    BaseType::named(type_name)
}
