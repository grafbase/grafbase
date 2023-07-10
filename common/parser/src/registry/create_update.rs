use case::CaseExt;

use dynaql::registry::relations::MetaRelationKind;
use dynaql::registry::{self, InputObjectType, NamedType, Registry};
use dynaql::registry::{
    resolvers::dynamo_mutation::DynamoMutationResolver, resolvers::dynamo_querying::DynamoResolver,
    resolvers::Resolver, resolvers::ResolverType, variables::VariableResolveDefinition, MetaField, MetaInputValue,
};

use dynaql::AuthConfig;
use dynaql_parser::types::{BaseType, ObjectType, Type, TypeDefinition, TypeKind};
use grafbase::auth::Operations;

use crate::registry::names::{
    MetaNames, INPUT_ARG_BY, INPUT_ARG_INPUT, INPUT_FIELD_RELATION_CREATE, INPUT_FIELD_RELATION_LINK,
    INPUT_FIELD_RELATION_UNLINK,
};
use crate::registry::ParentRelation;
use crate::rules::default_directive::DefaultDirective;

use crate::rules::model_directive::ModelDirective;
use crate::rules::relations::RelationEngine;
use crate::rules::resolver_directive::ResolverDirective;
use crate::rules::visitor::VisitorContext;
use crate::type_names::TypeNameExt;
use crate::utils::{to_input_type, to_lower_camelcase};

use super::names::{INPUT_FIELD_NUM_OP_DECREMENT, INPUT_FIELD_NUM_OP_INCREMENT, INPUT_FIELD_NUM_OP_SET};

/// Creates the create mutation and all relevant input/output types for the model. Given this
/// schema:
///
/// ```graphql
/// type Post {
///   id: ID!
///   content: String!
///   comments: [Comment] @relation(name: "comments")
///   ...
/// }
///
/// type Comment {
///   id: ID!
///   content: String!
///   post: Post @relation(name: "published")
/// }
/// ```
///
/// it would create something like:
///
/// ```graphql
/// """
/// This is the Comment Input type without the `commented` relation for the `Author`
/// """
/// type PostCommentsCommentCreateRelationInput {
///   create: PostCommentedCommentCreateInput
///   link: ID
/// }
///
/// type PostCommentsCommentCreateInput {
///    content: String!
/// }
///
/// """
/// Post create Input type
/// """
/// input PostCreateInput {
///   content: String!
///   comments: [PostCommentsCommentCreateRelationInput]
/// }
///
/// type Mutation {
///   postCreate(input: PostCreateInput): PostPayload
/// }
/// ```
///
pub fn add_mutation_create<'a>(
    ctx: &mut VisitorContext<'a>,
    model_type_definition: &'a TypeDefinition,
    object: &ObjectType,
    model_auth: Option<&AuthConfig>,
) {
    let type_name = MetaNames::model(model_type_definition);
    let input_type = register_input(
        ctx,
        &mut ctx.registry.borrow_mut(),
        model_type_definition,
        object,
        MutationKind::Create,
    );

    let create_payload = register_payload(ctx, model_type_definition, MutationKind::Create, model_auth);
    ctx.mutations.push(MetaField {
        name: MetaNames::mutation_create(model_type_definition),
        description: Some(format!("Create a {type_name}")),
        args: [MetaInputValue::new(INPUT_ARG_INPUT, format!("{input_type}!"))]
            .into_iter()
            .map(|input| (input.name.clone(), input))
            .collect(),
        ty: create_payload.as_nullable().into(),
        resolve: Some(Resolver {
            id: Some(format!("{}_create_resolver", type_name.to_lowercase())),
            r#type: ResolverType::DynamoMutationResolver(DynamoMutationResolver::CreateNode {
                input: VariableResolveDefinition::InputTypeName(INPUT_ARG_INPUT.to_owned()),
                ty: type_name.clone().into(),
            }),
        }),
        required_operation: Some(Operations::CREATE),
        auth: model_auth.cloned(),
        ..Default::default()
    });

    let batch_input_type = register_many_input(ctx, model_type_definition, MutationKind::Create, input_type);
    let batch_create_payload = register_many_payload(ctx, model_type_definition, MutationKind::Create, model_auth);
    ctx.mutations.push(MetaField {
        name: MetaNames::mutation_create_many(model_type_definition),
        description: Some(format!("Create multiple {type_name}")),
        args: [MetaInputValue::new(INPUT_ARG_INPUT, format!("[{batch_input_type}!]!"))]
            .into_iter()
            .map(|input| (input.name.clone(), input))
            .collect(),
        ty: batch_create_payload.as_nullable().into(),
        resolve: Some(Resolver {
            id: None,
            r#type: ResolverType::DynamoMutationResolver(DynamoMutationResolver::CreateNodes {
                input: VariableResolveDefinition::InputTypeName(INPUT_ARG_INPUT.to_owned()),
                ty: type_name.into(),
            }),
        }),
        required_operation: Some(Operations::CREATE),
        auth: model_auth.cloned(),
        ..Default::default()
    });
}

/// Creates the update mutation and all relevant input/output types for the model. Given this
/// schema:
///
/// ```graphql
/// type Post {
///   id: ID!
///   content: String!
///   comments: [Comment] @relation(name: "comments")
///   ...
/// }
///
/// type Comment {
///   id: ID!
///   content: String!
///   post: Post @relation(name: "published")
/// }
/// ```
///
/// it would create something like:
///
/// ```graphql
/// """
/// This is the Comment Input type without the `commented` relation for the `Author`
/// """
/// type PostCommentsCommentUpdateRelationInput {
///   create: PostCommentedCommentCreateInput
///   link: ID
///   unlink: ID
/// }
///
/// type PostCommentsCommentCreateInput {
///    content: String
/// }
///
/// """
/// Post create Input type
/// """
/// input PostUpdateInput {
///   content: String
///   comments: [PostCommentsCommentCreateRelationInput]
/// }
///
/// type Mutation {
///   postCreate(input: PostCreateInput): PostPayload
/// }
/// ```
///
pub fn add_mutation_update<'a>(
    ctx: &mut VisitorContext<'a>,
    model_type_definition: &'a TypeDefinition,
    object: &ObjectType,
    model_auth: Option<&AuthConfig>,
) {
    let type_name = MetaNames::model(model_type_definition);
    let input_type = register_input(
        ctx,
        &mut ctx.registry.borrow_mut(),
        model_type_definition,
        object,
        MutationKind::Update,
    );

    let update_payload = register_payload(ctx, model_type_definition, MutationKind::Update, model_auth);
    ctx.mutations.push(MetaField {
        name: MetaNames::mutation_update(model_type_definition),
        description: Some(format!("Update a {type_name}")),
        args: [
            MetaInputValue::new(INPUT_ARG_BY, format!("{}!", MetaNames::by_input(model_type_definition))),
            MetaInputValue::new(INPUT_ARG_INPUT, format!("{input_type}!")),
        ]
        .into_iter()
        .map(|input| (input.name.clone(), input))
        .collect(),
        ty: update_payload.as_nullable().into(),
        resolve: Some(Resolver {
            id: Some(format!("{}_create_resolver", type_name.to_lowercase())),
            r#type: ResolverType::DynamoMutationResolver(DynamoMutationResolver::UpdateNode {
                by: VariableResolveDefinition::InputTypeName(INPUT_ARG_BY.to_owned()),
                input: VariableResolveDefinition::InputTypeName(INPUT_ARG_INPUT.to_owned()),
                ty: type_name.clone().into(),
            }),
        }),
        required_operation: Some(Operations::UPDATE),
        auth: model_auth.cloned(),
        ..Default::default()
    });

    let batch_input_type = register_many_input(ctx, model_type_definition, MutationKind::Update, input_type);
    let batch_update_payload = register_many_payload(ctx, model_type_definition, MutationKind::Update, model_auth);
    ctx.mutations.push(MetaField {
        name: MetaNames::mutation_update_many(model_type_definition),
        description: Some(format!("Update multiple {type_name}")),
        args: [MetaInputValue::new(INPUT_ARG_INPUT, format!("[{batch_input_type}!]!"))]
            .into_iter()
            .map(|input| (input.name.clone(), input))
            .collect(),
        ty: batch_update_payload.as_nullable().into(),
        resolve: Some(Resolver {
            id: None,
            r#type: ResolverType::DynamoMutationResolver(DynamoMutationResolver::UpdateNodes {
                input: VariableResolveDefinition::InputTypeName(INPUT_ARG_INPUT.to_owned()),
                ty: type_name.into(),
            }),
        }),
        transformer: None,
        required_operation: Some(Operations::UPDATE),
        auth: model_auth.cloned(),
        ..Default::default()
    });
}

/// Used to define the input/output of the mutations:
/// - inputs: naming, nullable fields for updates, how relation should be treated, etc.
/// - outputs: naming
enum MutationKind<'a> {
    Create,
    Update,
    CreateOrLinkRelation(ParentRelation<'a>),
    CreateOrLinkOrUnlinkRelation(ParentRelation<'a>),
}

impl<'a> MutationKind<'a> {
    fn maybe_parent_relation(&self) -> Option<&ParentRelation<'a>> {
        match &self {
            Self::CreateOrLinkRelation(parent_relation) | Self::CreateOrLinkOrUnlinkRelation(parent_relation) => {
                Some(parent_relation)
            }
            Self::Create | Self::Update => None,
        }
    }

    fn nested<'b>(&self, parent_relation: ParentRelation<'b>) -> MutationKind<'b> {
        match &self {
            Self::Update => MutationKind::CreateOrLinkOrUnlinkRelation(parent_relation),
            Self::Create | Self::CreateOrLinkRelation(_) | Self::CreateOrLinkOrUnlinkRelation(_) => {
                MutationKind::CreateOrLinkRelation(parent_relation)
            }
        }
    }

    fn is_update(&self) -> bool {
        match self {
            Self::Update => true,
            // Deliberately not using '_' to ensure any potential new addition to MutationKind is
            // carefully thought through.
            Self::Create | Self::CreateOrLinkRelation(_) | Self::CreateOrLinkOrUnlinkRelation(_) => false,
        }
    }
}

fn register_many_input(
    ctx: &mut VisitorContext<'_>,
    model_type_definition: &TypeDefinition,
    mutation_kind: MutationKind<'_>,
    single_input_type: BaseType,
) -> BaseType {
    let input_type_name = if mutation_kind.is_update() {
        MetaNames::update_many_input(model_type_definition)
    } else {
        MetaNames::create_many_input(model_type_definition)
    };

    ctx.registry.borrow_mut().create_type(
        |_| {
            InputObjectType::new(input_type_name.clone(), {
                let mut args = Vec::new();
                if mutation_kind.is_update() {
                    args.push(MetaInputValue::new(
                        INPUT_ARG_BY,
                        format!("{}!", MetaNames::by_input(model_type_definition)),
                    ));
                }
                args.push(MetaInputValue::new(INPUT_ARG_INPUT, format!("{single_input_type}!")));
                args
            })
            .into()
        },
        &input_type_name,
        &input_type_name,
    );

    BaseType::named(&input_type_name)
}

/// Creates the actual input types.
/// See `add_mutation_create` and `add_mutation_update` for examples.
fn register_input(
    ctx: &VisitorContext<'_>,
    registry: &mut Registry,
    model_type_definition: &TypeDefinition,
    object: &ObjectType,
    mutation_kind: MutationKind<'_>,
) -> BaseType {
    let input_type_name: String = if mutation_kind.is_update() {
        MetaNames::update_input(model_type_definition)
    } else {
        MetaNames::create_input(model_type_definition, mutation_kind.maybe_parent_relation())
    };

    // type is only created if necessary
    registry.create_type(
        |registry| {
            let mut input_fields = vec![];
            let maybe_parent_relation_meta = mutation_kind
                .maybe_parent_relation()
                .map(|parent_relation| &parent_relation.meta);
            for field in object
                .fields
                .iter()
                .filter(|field| ModelDirective::is_not_metadata_field(field))
                .filter(|field| ResolverDirective::resolver_name(&field.node).is_none())
            {
                let maybe_type = match RelationEngine::get(ctx, &model_type_definition.name.node, &field.node) {
                    // If a relation exists, the field is a model
                    Some(model_to_field_relation) => {
                        ModelDirective::get_model_type_definition(ctx, &field.node.ty.node.base)
                            .filter(|_| {
                                // If parent relation exists we may skip the field we already have a value for it
                                maybe_parent_relation_meta
                                    .as_ref()
                                    .map(|parent_to_model_relation| {
                                        // Keep if different relation
                                        parent_to_model_relation.name != model_to_field_relation.name
                                            || (
                                                // Keep only if multiple parents can exist
                                                parent_to_model_relation.kind == MetaRelationKind::ManyToOne
                                                    || parent_to_model_relation.kind == MetaRelationKind::ManyToMany
                                            )
                                    })
                                    // Keep if no parent
                                    .unwrap_or(true)
                            })
                            .and_then(|pos_type_definition| {
                                let field_model_type_definition = &pos_type_definition.node;
                                // Actually it will always be an object. But currently the type system cannot express this.
                                match &field_model_type_definition.kind {
                                    TypeKind::Object(field_object) => {
                                        let field_input_base_type = register_input(
                                            ctx,
                                            registry,
                                            field_model_type_definition,
                                            field_object,
                                            mutation_kind.nested(ParentRelation {
                                                model_type_definition,
                                                meta: &model_to_field_relation,
                                            }),
                                        );
                                        // override base type while keeping the list and/or required parts
                                        Some(field.node.ty.node.override_base(field_input_base_type))
                                    }
                                    _ => None,
                                }
                            })
                    }
                    // field is not a model
                    None => Some(&field.node.ty.node.base.to_string())
                        // Type-specific overrides when updating
                        .filter(|_| mutation_kind.is_update())
                        .and_then(|base| match base.as_str() {
                            "Int" => Some(Type::nullable(register_numerical_operations(
                                registry,
                                NumericFieldKind::Int,
                            ))),
                            "Float" => Some(Type::nullable(register_numerical_operations(
                                registry,
                                NumericFieldKind::Float,
                            ))),
                            _ => None,
                        })
                        .or_else(|| Some(to_input_type(&ctx.types, field.node.ty.node.clone()))),
                };

                // We typically won't have any input field to add for a OneToOne relation field
                if let Some(r#type) = maybe_type {
                    let field_name = &field.node.name.node;
                    input_fields.push(MetaInputValue {
                        name: field_name.to_string(),
                        description: field.node.description.clone().map(|x| x.node),
                        ty: (if mutation_kind.is_update() {
                            Type::nullable(r#type.base)
                        } else {
                            r#type
                        })
                        .to_string(),
                        validators: super::get_length_validator(&field.node).map(|val| vec![val]),
                        visible: None,
                        default_value: (if mutation_kind.is_update() {
                            None
                        } else {
                            DefaultDirective::default_value_of(&field.node)
                        }),
                        is_secret: false,
                        rename: None,
                    });
                };
            }
            InputObjectType::new(input_type_name.clone(), input_fields)
                .with_description({
                    let description = format!(
                        "Input to {} a {}",
                        if mutation_kind.is_update() { "update" } else { "create" },
                        model_type_definition.name.node.to_camel()
                    );
                    mutation_kind
                        .maybe_parent_relation()
                        .map(|parent_relation| format!("{description} for the {parent_relation}"))
                        .or(Some(description))
                })
                .into()
        },
        &input_type_name,
        &input_type_name,
    );

    match mutation_kind {
        // Relation within an update
        MutationKind::CreateOrLinkOrUnlinkRelation(parent_relation) => {
            let relation_input_type_name = MetaNames::update_relation_input(&parent_relation, model_type_definition);

            registry.create_type(
                |_| {
                    InputObjectType::new(
                        relation_input_type_name.clone(),
                        [
                            MetaInputValue::new(INPUT_FIELD_RELATION_CREATE, input_type_name),
                            MetaInputValue::new(INPUT_FIELD_RELATION_LINK, "ID"),
                            MetaInputValue::new(INPUT_FIELD_RELATION_UNLINK, "ID"),
                        ],
                    )
                    .with_description(Some(format!(
                        "Input to link/unlink to or create a {} for the {}",
                        MetaNames::model(model_type_definition),
                        parent_relation,
                    )))
                    .with_oneof(true)
                    .into()
                },
                &relation_input_type_name,
                &relation_input_type_name,
            );

            BaseType::named(&relation_input_type_name)
        }
        // Relation within a create
        MutationKind::CreateOrLinkRelation(parent_relation) => {
            let relation_input_type_name = MetaNames::create_relation_input(&parent_relation, model_type_definition);

            registry.create_type(
                |_| {
                    InputObjectType::new(
                        relation_input_type_name.clone(),
                        [
                            MetaInputValue::new(INPUT_FIELD_RELATION_CREATE, input_type_name.clone()),
                            MetaInputValue::new(INPUT_FIELD_RELATION_LINK, "ID"),
                        ],
                    )
                    .with_description(Some(format!(
                        "Input to link to or create a {} for the {}",
                        MetaNames::model(model_type_definition),
                        parent_relation,
                    )))
                    .with_oneof(true)
                    .into()
                },
                &relation_input_type_name,
                &relation_input_type_name,
            );

            BaseType::named(&relation_input_type_name)
        }
        _ => BaseType::named(&input_type_name),
    }
}

fn register_payload<'a>(
    ctx: &mut VisitorContext<'a>,
    model_type_definition: &TypeDefinition,
    mutation_kind: MutationKind<'a>,
    model_auth: Option<&AuthConfig>,
) -> NamedType<'static> {
    let payload_type_name = if mutation_kind.is_update() {
        MetaNames::update_payload_type(model_type_definition)
    } else {
        MetaNames::create_payload_type(model_type_definition)
    };

    ctx.registry.get_mut().create_type(
        |_| {
            registry::ObjectType::new(payload_type_name.clone(), {
                let model_type_name = model_type_definition.name.node.to_string();
                let name = to_lower_camelcase(&model_type_name);
                [MetaField {
                    name,
                    ty: MetaNames::model(model_type_definition).into(),
                    resolve: Some(Resolver {
                        id: Some(format!("{}_resolver", model_type_name.to_lowercase())),
                        // Single entity
                        r#type: ResolverType::DynamoResolver(DynamoResolver::QueryPKSK {
                            pk: VariableResolveDefinition::LocalData("id".to_string()),
                            sk: VariableResolveDefinition::LocalData("id".to_string()),
                            schema: None,
                        }),
                    }),
                    required_operation: Some(if mutation_kind.is_update() {
                        Operations::UPDATE
                    } else {
                        Operations::CREATE
                    }),
                    auth: model_auth.cloned(),
                    ..Default::default()
                }]
            })
            .into()
        },
        &payload_type_name,
        &payload_type_name,
    );

    payload_type_name.into()
}

fn register_many_payload<'a>(
    ctx: &mut VisitorContext<'a>,
    model_type_definition: &TypeDefinition,
    mutation_kind: MutationKind<'a>,
    model_auth: Option<&AuthConfig>,
) -> NamedType<'static> {
    let payload_type_name = if mutation_kind.is_update() {
        MetaNames::update_many_payload_type(model_type_definition)
    } else {
        MetaNames::create_many_payload_type(model_type_definition)
    };

    let type_name = MetaNames::model(model_type_definition);
    ctx.registry.get_mut().create_type(
        |_| {
            registry::ObjectType::new(
                payload_type_name.clone(),
                [MetaField {
                    name: to_lower_camelcase(MetaNames::collection(model_type_definition)),
                    ty: NamedType::from(type_name.clone())
                        .as_non_null()
                        .list()
                        .non_null()
                        .into(),
                    resolve: Some(Resolver {
                        id: None,
                        r#type: ResolverType::DynamoResolver(DynamoResolver::QueryIds {
                            ids: VariableResolveDefinition::LocalData("ids".to_string()),
                            type_name,
                        }),
                    }),
                    required_operation: Some(if mutation_kind.is_update() {
                        Operations::UPDATE
                    } else {
                        Operations::CREATE
                    }),
                    auth: model_auth.cloned(),
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

pub enum NumericFieldKind {
    Int,
    Float,
}

impl NumericFieldKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Int => "Int",
            Self::Float => "Float",
        }
    }

    // purposely kept separate to prevent misuse
    fn to_type_name(&self) -> String {
        match self {
            Self::Int => "Int".to_string(),
            Self::Float => "Float".to_string(),
        }
    }
}

pub fn register_numerical_operations(registry: &mut Registry, numerical_field_kind: NumericFieldKind) -> BaseType {
    let operation_input_type_name = MetaNames::numerical_operation_input(&numerical_field_kind);

    registry.create_type(
        |_| {
            InputObjectType::new(
                operation_input_type_name.clone(),
                [
                    MetaInputValue::new(INPUT_FIELD_NUM_OP_SET, numerical_field_kind.to_type_name()),
                    MetaInputValue::new(INPUT_FIELD_NUM_OP_INCREMENT, numerical_field_kind.to_type_name()),
                    MetaInputValue::new(INPUT_FIELD_NUM_OP_DECREMENT, numerical_field_kind.to_type_name()),
                ],
            )
            .with_description(Some(format!(
                "Possible operations for {} {} field",
                match numerical_field_kind {
                    NumericFieldKind::Int => "an",
                    NumericFieldKind::Float => "a",
                },
                numerical_field_kind.as_str()
            )))
            .with_oneof(true)
            .into()
        },
        &operation_input_type_name,
        &operation_input_type_name,
    );

    BaseType::named(&operation_input_type_name)
}
