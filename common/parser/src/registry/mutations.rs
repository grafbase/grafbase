use case::CaseExt;

use dynaql::indexmap::{indexmap, IndexMap};

use dynaql::registry::relations::MetaRelationKind;
use dynaql::registry::Registry;
use dynaql::registry::{
    resolvers::dynamo_mutation::DynamoMutationResolver, resolvers::dynamo_querying::DynamoResolver,
    resolvers::Resolver, resolvers::ResolverType, variables::VariableResolveDefinition, MetaField, MetaInputValue,
    MetaType,
};

use dynaql::{AuthConfig, Operations};
use dynaql_parser::types::{BaseType, ObjectType, Type, TypeDefinition, TypeKind};

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
    let input_base_type = register_mutation_input_type(
        ctx,
        &mut ctx.registry.borrow_mut(),
        model_type_definition,
        object,
        MutationKind::Create,
    );
    let payload_base_type =
        register_mutation_payload_type(ctx, model_type_definition, MutationKind::Create, model_auth);

    ctx.mutations.push(MetaField {
        name: MetaNames::mutation_create(model_type_definition),
        description: Some(format!("Create a {type_name}")),
        args: indexmap! {
            INPUT_ARG_INPUT.to_owned() => MetaInputValue::new(
                INPUT_ARG_INPUT.to_owned(),
                Type::required(input_base_type)
            )
        },
        ty: Type::nullable(payload_base_type).to_string(),
        deprecation: dynaql::registry::Deprecation::NoDeprecated,
        cache_control: Default::default(),
        external: false,
        provides: None,
        requires: None,
        visible: None,
        edges: Vec::new(),
        relation: None,
        compute_complexity: None,
        resolve: Some(Resolver {
            id: Some(format!("{}_create_resolver", type_name.to_lowercase())),
            r#type: ResolverType::DynamoMutationResolver(DynamoMutationResolver::CreateNode {
                input: VariableResolveDefinition::InputTypeName(INPUT_ARG_INPUT.to_owned()),
                ty: type_name,
            }),
        }),
        plan: None,
        transformer: None,
        required_operation: Some(Operations::CREATE),
        auth: model_auth.cloned(),
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
    let input_base_type = register_mutation_input_type(
        ctx,
        &mut ctx.registry.borrow_mut(),
        model_type_definition,
        object,
        MutationKind::Update,
    );
    let payload_base_type =
        register_mutation_payload_type(ctx, model_type_definition, MutationKind::Update, model_auth);

    ctx.mutations.push(MetaField {
        name: MetaNames::mutation_update(model_type_definition),
        description: Some(format!("Update a {type_name}")),
        args: indexmap! {
            INPUT_ARG_BY.to_owned() => MetaInputValue::new(
                    INPUT_ARG_BY,
                    format!("{type_name}ByInput!"),
                ),
            INPUT_ARG_INPUT.to_owned() => MetaInputValue::new(
                    INPUT_ARG_INPUT,
                    Type::required(input_base_type),
                )
        },
        ty: Type::nullable(payload_base_type).to_string(),
        deprecation: dynaql::registry::Deprecation::NoDeprecated,
        cache_control: Default::default(),
        external: false,
        provides: None,
        requires: None,
        visible: None,
        edges: Vec::new(),
        relation: None,
        compute_complexity: None,
        resolve: Some(Resolver {
            id: Some(format!("{}_create_resolver", type_name.to_lowercase())),
            r#type: ResolverType::DynamoMutationResolver(DynamoMutationResolver::UpdateNode {
                by: VariableResolveDefinition::InputTypeName(INPUT_ARG_BY.to_owned()),
                input: VariableResolveDefinition::InputTypeName(INPUT_ARG_INPUT.to_owned()),
                ty: type_name,
            }),
        }),
        plan: None,
        transformer: None,
        required_operation: Some(Operations::UPDATE),
        auth: model_auth.cloned(),
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
            // Deliberatly not using '_' to ensure any potential new addition to MutationKind is
            // carefully thought through.
            Self::Create | Self::CreateOrLinkRelation(_) | Self::CreateOrLinkOrUnlinkRelation(_) => false,
        }
    }
}

/// Creates the actual input types.
/// See `add_mutation_create` and `add_mutation_update` for examples.
fn register_mutation_input_type(
    ctx: &VisitorContext<'_>,
    registry: &mut Registry,
    model_type_definition: &TypeDefinition,
    object: &ObjectType,
    mutation_kind: MutationKind<'_>,
) -> BaseType {
    let input_type_name: String = match &mutation_kind {
        MutationKind::Update => MetaNames::update_input(model_type_definition),
        _ => MetaNames::create_input(model_type_definition, mutation_kind.maybe_parent_relation()),
    };

    // type is only created if necessary
    registry.create_type(
        |registry| {
            let mut input_fields = IndexMap::new();
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
                                        let field_input_base_type = register_mutation_input_type(
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
                    input_fields.insert(
                        field_name.to_string(),
                        MetaInputValue {
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
                            default_value: DefaultDirective::default_value_of(&field.node),
                            is_secret: false,
                            rename: None,
                        },
                    );
                };
            }
            MetaType::InputObject {
                name: input_type_name.clone(),
                description: {
                    let description = format!(
                        "Input to {} a {}",
                        if mutation_kind.is_update() { "update" } else { "create" },
                        model_type_definition.name.node.to_camel()
                    );
                    mutation_kind
                        .maybe_parent_relation()
                        .map(|parent_relation| format!("{description} for the {parent_relation}"))
                        .or(Some(description))
                },
                oneof: false,
                input_fields,
                visible: None,
                rust_typename: input_type_name.clone(),
            }
        },
        &input_type_name,
        &input_type_name,
    );

    match mutation_kind {
        // Relation within an update
        MutationKind::CreateOrLinkOrUnlinkRelation(parent_relation) => {
            let relation_input_type_name = MetaNames::update_relation_input(&parent_relation, model_type_definition);

            registry.create_type(
                |_| MetaType::InputObject {
                    name: relation_input_type_name.clone(),
                    description: Some(format!(
                        "Input to link/unlink to or create a {} for the {}",
                        MetaNames::model(model_type_definition),
                        parent_relation,
                    )),
                    oneof: true,
                    input_fields: indexmap! {
                        INPUT_FIELD_RELATION_CREATE.to_string() => MetaInputValue::new(
                            INPUT_FIELD_RELATION_CREATE, input_type_name
                        ),
                        INPUT_FIELD_RELATION_LINK.to_string() => MetaInputValue::new(
                            INPUT_FIELD_RELATION_LINK, "ID"
                        ),
                        INPUT_FIELD_RELATION_UNLINK.to_string() => MetaInputValue::new(
                            INPUT_FIELD_RELATION_UNLINK, "ID"
                        ),
                    },
                    visible: None,
                    rust_typename: relation_input_type_name.clone(),
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
                |_| MetaType::InputObject {
                    name: relation_input_type_name.clone(),
                    description: Some(format!(
                        "Input to link to or create a {} for the {}",
                        MetaNames::model(model_type_definition),
                        parent_relation,
                    )),
                    oneof: true,
                    input_fields: indexmap! {
                        INPUT_FIELD_RELATION_CREATE.to_string() => MetaInputValue::new(
                            INPUT_FIELD_RELATION_CREATE,
                            input_type_name.clone(),
                        ),
                        INPUT_FIELD_RELATION_LINK.to_string() => MetaInputValue::new(
                            INPUT_FIELD_RELATION_LINK,
                            "ID",
                        ),
                    },
                    visible: None,
                    rust_typename: relation_input_type_name.clone(),
                },
                &relation_input_type_name,
                &relation_input_type_name,
            );

            BaseType::named(&relation_input_type_name)
        }
        _ => BaseType::named(&input_type_name),
    }
}

fn register_mutation_payload_type<'a>(
    ctx: &mut VisitorContext<'a>,
    model_type_definition: &TypeDefinition,
    mutation_kind: MutationKind<'a>,
    model_auth: Option<&AuthConfig>,
) -> BaseType {
    let payload_type_name = if mutation_kind.is_update() {
        MetaNames::update_payload_type(model_type_definition)
    } else {
        MetaNames::create_payload_type(model_type_definition)
    };

    ctx.registry.get_mut().create_type(
        |_| MetaType::Object {
            name: payload_type_name.clone(),
            description: None,
            fields: {
                let model_type_name = model_type_definition.name.node.to_string();
                let name = to_lower_camelcase(&model_type_name);
                indexmap! {
                    name.clone() =>  MetaField {
                        name,
                        description: None,
                        args: Default::default(),
                        ty: MetaNames::model(model_type_definition),
                        deprecation: Default::default(),
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None,
                        visible: None,
                        compute_complexity: None,
                        edges: Vec::new(),
                        relation: None,
                        resolve: Some(Resolver {
                            id: Some(format!("{}_resolver", model_type_name.to_lowercase())),
                            // Single entity
                            r#type: ResolverType::DynamoResolver(DynamoResolver::QueryPKSK {
                                pk: VariableResolveDefinition::LocalData("id".to_string()),
                                sk: VariableResolveDefinition::LocalData("id".to_string()),
                                schema: None,
                            }),
                        }),
                        plan: None,
                        transformer: None,
                        required_operation: Some(if mutation_kind.is_update() { Operations::UPDATE } else {Operations::CREATE}),
                        auth: model_auth.cloned(),
                    },
                }
            },
            cache_control: Default::default(),
            extends: false,
            keys: None,
            visible: None,
            is_subscription: false,
            is_node: false,
            rust_typename: payload_type_name.clone(),
            constraints: vec![],
        },
        &payload_type_name,
        &payload_type_name,
    );

    BaseType::named(&payload_type_name)
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
        |_| MetaType::InputObject {
            name: operation_input_type_name.clone(),
            description: Some(format!(
                "Possible operations for {} {} field",
                match numerical_field_kind {
                    NumericFieldKind::Int => "an",
                    NumericFieldKind::Float => "a",
                },
                numerical_field_kind.as_str()
            )),
            oneof: true,
            input_fields: IndexMap::from([
                (
                    INPUT_FIELD_NUM_OP_SET.to_string(),
                    MetaInputValue::new(INPUT_FIELD_NUM_OP_SET, numerical_field_kind.to_type_name()),
                ),
                (
                    INPUT_FIELD_NUM_OP_INCREMENT.to_string(),
                    MetaInputValue::new(INPUT_FIELD_NUM_OP_INCREMENT, numerical_field_kind.to_type_name()),
                ),
                (
                    INPUT_FIELD_NUM_OP_DECREMENT.to_string(),
                    MetaInputValue::new(INPUT_FIELD_NUM_OP_DECREMENT, numerical_field_kind.to_type_name()),
                ),
            ]),
            visible: None,
            rust_typename: operation_input_type_name.clone(),
        },
        &operation_input_type_name,
        &operation_input_type_name,
    );

    BaseType::named(&operation_input_type_name)
}
