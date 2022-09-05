use crate::rules::relations::generate_metarelation;
use crate::rules::visitor::VisitorContext;
use crate::utils::{is_modelized_node, to_base_type_str, to_defined_input_type, to_input_type, to_lower_camelcase};
use case::CaseExt;
use dynaql::indexmap::IndexMap;
use dynaql::registry::relations::MetaRelation;
use dynaql::registry::{
    resolvers::dynamo_mutation::DynamoMutationResolver, resolvers::dynamo_querying::DynamoResolver,
    resolvers::Resolver, resolvers::ResolverType, variables::VariableResolveDefinition, MetaField, MetaInputValue,
    MetaType,
};
use dynaql::Operations;
use dynaql::Positioned;
use dynaql_parser::types::{FieldDefinition, ObjectType, TypeDefinition, TypeKind};

/// Create an input type for a Node's Relation Create.
///
/// ```graphql
/// input PostPublishedAuthorCreateInput {
///   create: ...AuthorWithoutPublishedRelation
///   link: ID
/// }
/// ```
fn create_input_relation<'a>(
    ctx: &mut VisitorContext<'a>,
    ty_from: &TypeDefinition,
    ty_to: &TypeDefinition,
    relation: &MetaRelation,
    field: &Positioned<FieldDefinition>,
) -> String {
    let ty_from_name = ty_from.name.node.to_camel();
    let ty_to_name = ty_to.name.node.to_camel();

    let prefix = format!(
        "{}{}{}",
        ty_from_name.to_camel(),
        relation.name.to_camel(),
        ty_to_name.to_camel(),
    );

    let input_name = format!("{prefix}CreateInput");

    if ctx.types.get(&input_name).is_some() {
        return input_name;
    }

    match &ty_to.kind {
        TypeKind::Object(object) => {
            let mut input_fields = IndexMap::new();
            for field in &object.fields {
                let name = &field.node.name.node;
                // If it's a modelized node, we want to generate
                let actual_field_type = is_modelized_node(&ctx.types, &field.node.ty.node);

                if actual_field_type.is_some() {
                    let relation_name = generate_metarelation(ty_to, &field.node).name;
                    if relation_name == relation.name {
                        // If we are in the same relation we try to reverse
                        continue;
                    }

                    let field_base_ty = to_base_type_str(&field.node.ty.node.base);
                    let input_name = format!(
                        "{}{}{}CreateInput",
                        &ty_to.name.node.to_camel(),
                        relation_name.clone().to_camel(),
                        field_base_ty.to_camel()
                    );

                    input_fields.insert(
                        name.to_string(),
                        MetaInputValue {
                            name: name.to_string(),
                            description: field.node.description.clone().map(|x| x.node),
                            ty: to_defined_input_type(field.node.ty.clone().node, input_name).to_string(),
                            visible: None,
                            default_value: None,
                            is_secret: false,
                        },
                    );
                    continue;
                }

                // TODO: Abstract this behind an `ID` utility;
                if name.ne("id") {
                    input_fields.insert(
                        name.clone().to_string(),
                        MetaInputValue {
                            name: name.to_string(),
                            description: field.node.description.clone().map(|x| x.node),
                            ty: to_input_type(&ctx.types, field.node.ty.clone().node).to_string(),
                            visible: None,
                            default_value: None,
                            is_secret: false,
                        },
                    );
                    continue;
                }
            }

            ctx.registry.get_mut().create_type(
                &mut |_| MetaType::InputObject {
                    name: input_name.clone(),
                    description: Some(format!("Input to create a new {prefix}")),
                    oneof: false,
                    input_fields: input_fields.clone(),
                    visible: None,
                    rust_typename: input_name.clone(),
                },
                &input_name,
                &input_name,
            );
        }
        _ => ctx.report_error(
            vec![field.pos],
            format!(
                "You should have an `Object` type here for field: `{}` in `{}`.",
                field.node.name.node, &ty_from.name.node
            ),
        ),
    }

    let input_name_link = format!("{prefix}CreateRelationInput");

    ctx.registry.get_mut().create_type(
        &mut |_| MetaType::InputObject {
            name: input_name_link.clone(),
            description: Some(format!("Input to create a new {prefix} relation")),
            oneof: true,
            input_fields: {
                let mut input_fields = IndexMap::new();

                input_fields.insert(
                    "create".to_string(),
                    MetaInputValue {
                        name: "create".to_string(),
                        description: None,
                        ty: input_name.clone(),
                        visible: None,
                        default_value: None,
                        is_secret: false,
                    },
                );

                input_fields.insert(
                    "link".to_string(),
                    MetaInputValue {
                        name: "link".to_string(),
                        description: None,
                        ty: "ID".to_string(),
                        visible: None,
                        default_value: None,
                        is_secret: false,
                    },
                );

                input_fields
            },
            visible: None,
            rust_typename: input_name_link.clone(),
        },
        &input_name_link,
        &input_name_link,
    );

    input_name_link
}

/// We do create the `input` type of every possibility for a Type.
///
/// For each `@modelized` directive, we want to create an `input` available
/// for each type.
///
/// # Example
///
/// ```graphql
/// type Post @modelized {
///   id: ID!
///   author: Author @relation(name: "published")
///   comments: [Comment] @relation(name: "comments")
///   ...
/// }
/// ```
///
/// Would create
///
/// ```graphql
/// input PostPublishedAuthorCreateRelationInput {...}
/// input PostCommentsCommentCreateRelationInput {...}
/// ```
pub fn create_input_without_relation<'a>(ctx: &mut VisitorContext<'a>, ty: &TypeDefinition, object: &ObjectType) {
    let type_name = ty.name.node.to_camel();
    let create_input_name = format!("{}CreateInput", type_name);
    let mut input_fields = IndexMap::new();

    if ctx.types.get(&create_input_name).is_some() {
        return;
    }

    for field in &object.fields {
        let name = &field.node.name.node;
        // If it's a modelized node, we want to generate
        let types = ctx.types.clone(); // TODO: We should change a little the way it works, this clone can be avoided, not really expensive but should still be reworked.

        let actual_field_type = is_modelized_node(&types, &field.node.ty.node);
        if let Some(ty_to) = actual_field_type {
            // Should trigger the creation of the sub input
            let relation = generate_metarelation(ty, &field.node);
            let input_name = create_input_relation(ctx, ty, &ty_to.node, &relation, field);

            input_fields.insert(
                name.to_string(),
                MetaInputValue {
                    name: name.to_string(),
                    description: field.node.description.clone().map(|x| x.node),
                    ty: to_defined_input_type(field.node.ty.clone().node, input_name).to_string(),
                    visible: None,
                    default_value: None,
                    is_secret: false,
                },
            );
            continue;
        }

        // TODO: Abstract this behind an `ID` utility;
        if name.ne("id") {
            input_fields.insert(
                name.clone().to_string(),
                MetaInputValue {
                    name: name.to_string(),
                    description: field.node.description.clone().map(|x| x.node),
                    ty: to_input_type(&ctx.types, field.node.ty.clone().node).to_string(),
                    visible: None,
                    default_value: None,
                    is_secret: false,
                },
            );
            continue;
        }
    }

    ctx.registry.get_mut().create_type(
        &mut |_| MetaType::InputObject {
            name: create_input_name.clone(),
            description: Some(format!("Input to create a new {}", &type_name)),
            oneof: false,
            input_fields: input_fields.clone(),
            visible: None,
            rust_typename: type_name.clone(),
        },
        &create_input_name,
        &create_input_name,
    );
}

/// The idea there is to generate the create mutation of an Entity depending on
/// the fields of the Entity. If it's linked to another Node based on a relation
/// we'll generate an Input based on a `link` or a `create`.
///
/// ```graphql
/// type Post {
///   id: ID!
///   content: String
///   author: Author @relation(name: "published")
///   comments: [Comment] @relation(name: "comments")
///   ...
/// }
///
/// type Author {
///   id: ID!
///   firstName: String
///   lastName: String
///   postPublished: [Post] @relation(name: "published")
///   comments: [Comment] @relation(name: "commented")
/// }
///
/// type Comment {
///   id: ID!
///   content: String
///   post: Post @relation(name: "published")
///   author: Author @relation(name: "commented")
/// }
/// ```
///
/// Would create something like:
///
/// ```graphql
/// input PostPublishedCreateRelationInput {
///   """
///   As the relation `published` got an edge from `Author` to `Post`, we do not
///   need to have this relation available on the creation.
///   """
///   create: PostPublishedAuthorCreateRelationInput
///   link: ID
/// }
///
/// """
/// This is the Author Input type without the `published` relation for the `Post`
/// """
/// type PostPublishedAuthorCreateRelationInput {
///   firstName: String
///   lastName: String
///   comments: [AuthorCommentedCommentCreateRelationInput]
/// }
///
/// """
/// This is the Comment Input type without the `commented` relation for the `Author`
/// """
/// type AuthorCommentedCommentCreateRelationInput {
///   content: String
///   post: CommentPublishedPostCreateRelationInput
/// }
///
/// type PostCommentCreateRelationInput {
/// }
///
/// """
/// Post create Input type
/// """
/// input PostCreateInput {
///   content: String
///   author: PostPublishedCreateRelationInput
///   comments: [PostCommentCreateRelationInput]
/// }
///
/// type Mutation {
///   postCreate(input: PostCreateInput): PostPayload
/// }
/// ```
///
pub fn add_create_mutation<'a>(
    ctx: &mut VisitorContext<'a>,
    ty: &TypeDefinition,
    object: &ObjectType,
    type_name: &str,
) {
    create_input_without_relation(ctx, ty, object);
    let type_name = type_name.to_string();
    let create_input_name = format!("{}CreateInput", type_name.to_camel());

    let create_payload_name = format!("{}CreatePayload", type_name.to_camel());
    // CreatePayload
    ctx.registry.get_mut().create_type(
        &mut |_| MetaType::Object {
            name: create_payload_name.clone(),
            description: None,
            fields: {
                let mut fields = IndexMap::new();
                let name = to_lower_camelcase(&type_name);
                fields.insert(
                    name.clone(),
                    MetaField {
                        name,
                        description: None,
                        args: Default::default(),
                        ty: type_name.to_camel(),
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
                            id: Some(format!("{}_resolver", type_name.to_lowercase())),
                            // Single entity
                            r#type: ResolverType::DynamoResolver(DynamoResolver::QueryPKSK {
                                pk: VariableResolveDefinition::LocalData("id".to_string()),
                                sk: VariableResolveDefinition::LocalData("id".to_string()),
                            }),
                        }),
                        transforms: None,
                        required_operation: Some(Operations::CREATE),
                    },
                );
                fields
            },
            cache_control: dynaql::CacheControl {
                public: true,
                max_age: 0usize,
            },
            extends: false,
            keys: None,
            visible: None,
            is_subscription: false,
            is_node: false,
            rust_typename: create_payload_name.clone(),
            constraints: vec![],
        },
        &create_payload_name,
        &create_payload_name,
    );

    // createQuery
    ctx.mutations.push(MetaField {
        name: format!("{}Create", to_lower_camelcase(&type_name)),
        description: Some(format!("Create a {}", type_name)),
        args: {
            let mut args = IndexMap::new();
            args.insert(
                "input".to_owned(),
                MetaInputValue {
                    name: "input".to_owned(),
                    description: None,
                    ty: format!("{}!", &create_input_name),
                    default_value: None,
                    visible: None,
                    is_secret: false,
                },
            );
            args
        },
        ty: create_payload_name,
        deprecation: dynaql::registry::Deprecation::NoDeprecated,
        cache_control: dynaql::CacheControl {
            public: true,
            max_age: 0usize,
        },
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
                input: VariableResolveDefinition::InputTypeName("input".to_owned()),
                ty: type_name,
            }),
        }),
        transforms: None,
        required_operation: Some(Operations::CREATE),
    });
}

/*
 * TODO: Fix this
#[cfg(test)]
mod tests {
    use dynaql::{Name, Pos, Positioned, Schema};
    use dynaql_parser::types::{FieldDefinition, ObjectType, ServiceDocument, Type, TypeDefinition};
    use insta::assert_snapshot;

    use crate::rules::visitor::VisitorContext;

    use super::add_create_mutation;

    #[test(skip)]
    fn ensure_create_mutation_types() {
        let doc = ServiceDocument { definitions: vec![] };
        let mut ctx = VisitorContext::new(&doc);
        let field_definition = FieldDefinition {
            ty: Positioned::new(Type::new("Author").unwrap(), Pos::default()),
            description: None,
            name: Positioned::new(Name::new("Author"), Pos::default()),
            arguments: Vec::new(),
            directives: Vec::new(),
        };

        let fake_object_ty = ObjectType {
            fields: vec![Positioned {
                pos: Pos { line: 3, column: 4 },
                node: field_definition,
            }],
            implements: Vec::new(),
        };

        let fake_object_type_def = TypeDefinition {
            kind: dynaql_parser::types::TypeKind::Object(fake_object_ty.clone()),
            description: None,
            name: Positioned {
                pos: Pos { line: 1, column: 2 },
                node: Name::new("Author"),
            },
            directives: Vec::new(),
            extend: false,
        };

        add_create_mutation(&mut ctx, &fake_object_type_def, &fake_object_ty, "Author");
        let sdl = Schema::new(ctx.registry.take()).sdl();
        assert_snapshot!(sdl);
    }
}
*/
