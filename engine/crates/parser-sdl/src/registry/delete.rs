use grafbase_engine::{
    names::{OUTPUT_FIELD_DELETED_ID, OUTPUT_FIELD_DELETED_IDS},
    registry::{
        resolvers::{dynamo_mutation::DynamoMutationResolver, transformer::Transformer},
        variables::VariableResolveDefinition,
        InputObjectType, MetaField, MetaInputValue, NamedType, ObjectType,
    },
    AuthConfig, CacheControl,
};
use grafbase_engine_parser::types::{BaseType, TypeDefinition};
use common_types::auth::Operations;

use crate::{
    registry::names::{MetaNames, INPUT_ARG_BY, INPUT_ARG_INPUT},
    rules::visitor::VisitorContext,
    type_names::TypeNameExt,
};

pub fn add_mutation_delete<'a>(
    ctx: &mut VisitorContext<'a>,
    model_type_definition: &'a TypeDefinition,
    model_auth: Option<&AuthConfig>,
    cache_control: CacheControl,
) {
    let type_name = MetaNames::model(model_type_definition);
    let payload = register_payload(ctx, model_type_definition, model_auth, cache_control.clone());

    // deleteMutation
    ctx.mutations.push(MetaField {
        name: MetaNames::mutation_delete(model_type_definition),
        description: Some(format!("Delete a {type_name} by ID or unique field")),
        args: [MetaInputValue::new(
            INPUT_ARG_BY,
            format!("{}!", MetaNames::by_input(model_type_definition)),
        )]
        .into_iter()
        .map(|input| (input.name.clone(), input))
        .collect(),
        ty: payload.as_nullable().into(),
        resolver: DynamoMutationResolver::DeleteNode {
            ty: type_name.clone().into(),
            by: VariableResolveDefinition::input_type_name(INPUT_ARG_BY),
        }
        .into(),
        cache_control: cache_control.clone(),
        required_operation: Some(Operations::DELETE),
        auth: model_auth.cloned(),
        ..Default::default()
    });

    let many_input_type = register_many_input(ctx, model_type_definition);
    let delete_many_payload = register_many_payload(ctx, model_type_definition, model_auth, cache_control.clone());
    ctx.mutations.push(MetaField {
        name: MetaNames::mutation_delete_many(model_type_definition),
        description: Some(format!("Delete multiple {type_name}")),
        args: [MetaInputValue::new(INPUT_ARG_INPUT, format!("[{many_input_type}!]!"))]
            .into_iter()
            .map(|input| (input.name.clone(), input))
            .collect(),
        ty: delete_many_payload.as_nullable().into(),
        resolver: DynamoMutationResolver::DeleteNodes {
            input: VariableResolveDefinition::input_type_name(INPUT_ARG_INPUT),
            ty: type_name.into(),
        }
        .into(),
        cache_control,
        required_operation: Some(Operations::DELETE),
        auth: model_auth.cloned(),
        ..Default::default()
    });
}

fn register_many_input(ctx: &mut VisitorContext<'_>, model_type_definition: &TypeDefinition) -> BaseType {
    let input_type_name = MetaNames::delete_many_input(model_type_definition);

    ctx.registry.borrow_mut().create_type(
        |_| {
            InputObjectType::new(
                input_type_name.clone(),
                [MetaInputValue::new(
                    INPUT_ARG_BY,
                    format!("{}!", MetaNames::by_input(model_type_definition)),
                )],
            )
            .into()
        },
        &input_type_name,
        &input_type_name,
    );

    BaseType::named(&input_type_name)
}

fn register_payload(
    ctx: &mut VisitorContext<'_>,
    model_type_definition: &TypeDefinition,
    model_auth: Option<&AuthConfig>,
    cache_control: CacheControl,
) -> NamedType<'static> {
    let payload_type_name = MetaNames::delete_payload_type(model_type_definition);
    // DeletePayload
    ctx.registry.get_mut().create_type(
        |_| {
            ObjectType::new(
                payload_type_name.clone(),
                [MetaField {
                    name: OUTPUT_FIELD_DELETED_ID.to_string(),
                    ty: NamedType::from("ID").as_non_null().into(),
                    resolver: Transformer::select("id").into(),
                    required_operation: Some(Operations::DELETE),
                    auth: model_auth.cloned(),
                    cache_control: cache_control.clone(),
                    ..Default::default()
                }],
            )
            .into()
        },
        &payload_type_name,
        &payload_type_name,
    );

    payload_type_name.into()
}

fn register_many_payload(
    ctx: &mut VisitorContext<'_>,
    model_type_definition: &TypeDefinition,
    model_auth: Option<&AuthConfig>,
    cache_control: CacheControl,
) -> NamedType<'static> {
    let payload_type_name = MetaNames::delete_many_payload_type(model_type_definition);
    // DeletePayload
    ctx.registry.get_mut().create_type(
        |_| {
            ObjectType::new(
                payload_type_name.clone(),
                [MetaField {
                    name: OUTPUT_FIELD_DELETED_IDS.to_string(),
                    ty: NamedType::from("ID").as_non_null().list().non_null().into(),
                    resolver: Transformer::select("ids").into(),
                    required_operation: Some(Operations::DELETE),
                    auth: model_auth.cloned(),
                    cache_control: cache_control.clone(),
                    ..Default::default()
                }],
            )
            .into()
        },
        &payload_type_name,
        &payload_type_name,
    );

    payload_type_name.into()
}
