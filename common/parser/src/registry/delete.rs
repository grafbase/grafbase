use dynaql::registry::transformers::Transformer;
use dynaql::registry::{
    resolvers::dynamo_mutation::DynamoMutationResolver, resolvers::Resolver, resolvers::ResolverType,
    variables::VariableResolveDefinition, MetaField, MetaInputValue,
};
use dynaql::registry::{InputObjectType, NamedType, ObjectType};

use dynaql::{AuthConfig, CacheControl};
use dynaql_parser::types::{BaseType, TypeDefinition};
use grafbase::auth::Operations;

use crate::registry::names::{MetaNames, INPUT_ARG_BY, INPUT_ARG_INPUT};
use crate::rules::visitor::VisitorContext;
use crate::type_names::TypeNameExt;

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
        resolve: Some(Resolver {
            id: Some(format!("{}_delete_resolver", type_name.to_lowercase())),
            r#type: ResolverType::DynamoMutationResolver(DynamoMutationResolver::DeleteNode {
                ty: type_name.clone().into(),
                by: VariableResolveDefinition::InputTypeName(INPUT_ARG_BY.to_owned()),
            }),
        }),

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
        resolve: Some(Resolver {
            id: None,
            r#type: ResolverType::DynamoMutationResolver(DynamoMutationResolver::DeleteNodes {
                input: VariableResolveDefinition::InputTypeName(INPUT_ARG_INPUT.to_owned()),
                ty: type_name.into(),
            }),
        }),
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
                    name: "deletedId".to_string(),
                    ty: NamedType::from("ID").as_non_null().into(),
                    transformer: Some(Transformer::JSONSelect {
                        property: "id".to_string(),
                    }),
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
                    name: "deletedIds".to_string(),
                    ty: NamedType::from("ID").as_non_null().list().non_null().into(),
                    transformer: Some(Transformer::JSONSelect {
                        property: "ids".to_string(),
                    }),
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
