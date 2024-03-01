use case::CaseExt;
use common_types::auth::Operations;
use engine::{
    names::{MONGODB_OUTPUT_FIELD_ID, OUTPUT_FIELD_ID},
    registry::{
        resolvers::{transformer::Transformer, Resolver},
        InputObjectType, MetaField, MetaInputValue, ObjectType,
    },
};

use crate::{
    registry::{get_length_validator, names::MetaNames},
    rules::{
        default_directive::DefaultDirective,
        mongodb_directive::model_directive::create_type_context::CreateTypeContext, visitor::VisitorContext,
    },
    utils::to_input_type,
};

pub(crate) fn register_input(visitor_ctx: &mut VisitorContext<'_>, create_ctx: &CreateTypeContext<'_>) -> String {
    let input_type_name = MetaNames::create_input(create_ctx.r#type);

    let implicit_fields = std::iter::once({
        let mut input = MetaInputValue::new(OUTPUT_FIELD_ID, "ID");
        input.rename = Some(MONGODB_OUTPUT_FIELD_ID.to_string());

        input
    });

    let explicit_fields = create_ctx.object.fields.iter().map(|field| {
        let r#type = to_input_type(&visitor_ctx.types, field.r#type().clone());
        let mut input = MetaInputValue::new(field.node.name.node.to_string(), r#type.to_string());

        input.description = field.description().map(ToString::to_string);
        input.rename = field.mapped_name().map(ToString::to_string);
        input.default_value = DefaultDirective::default_value_of(field);
        input.validators = get_length_validator(field).map(|validator| vec![validator]);

        input
    });

    let input_fields = implicit_fields.chain(explicit_fields);
    let description = format!("Input to create a {}", create_ctx.model_name().to_camel());
    let input_type = InputObjectType::new(input_type_name.clone(), input_fields).with_description(Some(description));

    visitor_ctx
        .registry
        .borrow_mut()
        .create_type(|_| input_type.into(), &input_type_name, &input_type_name);

    input_type_name
}

pub(crate) fn register_single_output(
    visitor_ctx: &mut VisitorContext<'_>,
    create_ctx: &CreateTypeContext<'_>,
) -> String {
    let output_type_name = MetaNames::create_payload_type(create_ctx.r#type);
    let mut output_field = MetaField::new("insertedId", "ID");

    let transformer = Transformer::select("insertedId");
    output_field.resolver = Resolver::from(transformer);
    output_field.required_operation = Some(Operations::CREATE);
    output_field.auth = create_ctx.model_auth().clone();

    let object_type = ObjectType::new(&output_type_name, [output_field]);

    visitor_ctx
        .registry
        .get_mut()
        .create_type(|_| object_type.into(), &output_type_name, &output_type_name);

    output_type_name
}

pub(crate) fn register_many_output(visitor_ctx: &mut VisitorContext<'_>, create_ctx: &CreateTypeContext<'_>) -> String {
    let output_type_name = MetaNames::create_many_payload_type(create_ctx.r#type);
    let mut output_field = MetaField::new("insertedIds", "[ID]");

    let transformer = Transformer::select("insertedIds");
    output_field.resolver = Resolver::from(transformer);
    output_field.required_operation = Some(Operations::CREATE);
    output_field.auth = create_ctx.model_auth().clone();

    let object_type = ObjectType::new(&output_type_name, [output_field]);

    visitor_ctx
        .registry
        .get_mut()
        .create_type(|_| object_type.into(), &output_type_name, &output_type_name);

    output_type_name
}
