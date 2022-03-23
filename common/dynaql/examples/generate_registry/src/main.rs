use async_graphql::model::__Schema;
use async_graphql::registry::DebugResolver;
use async_graphql::registry::DynamoResolver;
use async_graphql::registry::MetaField;
use async_graphql::registry::MetaInputValue;
use async_graphql::registry::Registry;
use async_graphql::registry::Resolver;
use async_graphql::registry::ResolverType;
use async_graphql::registry::Transformer;
use async_graphql::OutputType;
use async_graphql::Schema;
use std::io::Write;
/*
 *
pub struct Account {
    #[dynomite(rename = "pk")]
    pub id: crate::AccountKeyId,
    // sk = pk
    pub sk: crate::AccountKeyId,
    pub slug: String,
    pub name: String,
    pub kind: crate::AccountKind,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

*/

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let mut registry = Schema::create_registry();
    registry.create_type(
        &mut |_| async_graphql::registry::MetaType::Object {
            name: "Account".to_owned(),
            description: None,
            fields: {
                let mut fields = async_graphql::indexmap::IndexMap::new();
                fields.insert(
                    "id".to_string(),
                    MetaField {
                        name: "id".to_string(),
                        description: None,
                        args: Default::default(),
                        ty: "ID!".to_string(),
                        deprecation: Default::default(),
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None,
                        visible: None,
                        compute_complexity: None,
                        resolve: None,
                        transforms: Some(vec![Transformer::DynamoSelect {
                            property: "pk".to_string(),
                        }]),
                    },
                );
                fields.insert(
                    "slug".to_string(),
                    MetaField {
                        name: "slug".to_string(),
                        description: None,
                        args: Default::default(),
                        ty: "String!".to_string(),
                        deprecation: Default::default(),
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None,
                        visible: None,
                        compute_complexity: None,
                        resolve: None,
                        transforms: Some(vec![Transformer::DynamoSelect {
                            property: "slug".to_string(),
                        }]),
                    },
                );
                fields.insert(
                    "name".to_string(),
                    MetaField {
                        name: "name".to_string(),
                        description: None,
                        args: Default::default(),
                        ty: "String!".to_string(),
                        deprecation: Default::default(),
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None,
                        visible: None,
                        compute_complexity: None,
                        resolve: None,
                        transforms: Some(vec![Transformer::DynamoSelect {
                            property: "name".to_string(),
                        }]),
                    },
                );
                fields
            },
            cache_control: async_graphql::CacheControl {
                public: true,
                max_age: 0usize,
            },
            extends: false,
            keys: ::std::option::Option::None,
            visible: ::std::option::Option::None,
            is_subscription: false,
            rust_typename: "User".to_owned(),
        },
        "Account",
        "Account",
    );

    // Add User type
    registry.create_type(
        &mut |_| async_graphql::registry::MetaType::Object {
            name: "User".to_owned(),
            description: None,
            fields: {
                let mut fields = async_graphql::indexmap::IndexMap::new();
                fields.insert(
                    "id".to_string(),
                    MetaField {
                        name: "id".to_string(),
                        description: None,
                        args: Default::default(),
                        ty: "ID!".to_string(),
                        deprecation: Default::default(),
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None,
                        visible: None,
                        compute_complexity: None,
                        resolve: None,
                        transforms: Some(vec![Transformer::DynamoSelect {
                            property: "pk".to_string(),
                        }]),
                    },
                );
                fields.insert(
                    "clerkUserId".to_string(),
                    MetaField {
                        name: "clerkUserId".to_string(),
                        description: None,
                        args: Default::default(),
                        ty: "ID!".to_string(),
                        deprecation: Default::default(),
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None,
                        visible: None,
                        compute_complexity: None,
                        resolve: None,
                        transforms: Some(vec![Transformer::DynamoSelect {
                            property: "clerk_user_id".to_string(),
                        }]),
                    },
                );
                fields.insert(
                    "name".to_string(),
                    MetaField {
                        name: "name".to_string(),
                        description: None,
                        args: Default::default(),
                        ty: "String!".to_string(),
                        deprecation: Default::default(),
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None,
                        visible: None,
                        compute_complexity: None,
                        resolve: None,
                        transforms: Some(vec![Transformer::DynamoSelect {
                            property: "name".to_string(),
                        }]),
                    },
                );
                fields.insert(
                    "email".to_string(),
                    MetaField {
                        name: "email".to_string(),
                        description: None,
                        args: Default::default(),
                        ty: "String!".to_string(),
                        deprecation: Default::default(),
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None,
                        visible: None,
                        compute_complexity: None,
                        resolve: None,
                        transforms: Some(vec![Transformer::DynamoSelect {
                            property: "email".to_string(),
                        }]),
                    },
                );
                fields.insert(
                    "avatar".to_string(),
                    MetaField {
                        name: "avatar".to_string(),
                        description: None,
                        args: Default::default(),
                        ty: "String".to_string(),
                        deprecation: Default::default(),
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None,
                        visible: None,
                        compute_complexity: None,
                        resolve: None,
                        transforms: Some(vec![Transformer::DynamoSelect {
                            property: "avatar".to_string(),
                        }]),
                    },
                );
                fields
            },
            cache_control: async_graphql::CacheControl {
                public: true,
                max_age: 0usize,
            },
            extends: false,
            keys: ::std::option::Option::None,
            visible: ::std::option::Option::None,
            is_subscription: false,
            rust_typename: "User".to_owned(),
        },
        "User",
        "User",
    );

    // Add Query type
    registry.create_type(
        &mut |registry| {
            let schema_type = __Schema::create_type_info(registry);
            async_graphql::registry::MetaType::Object {
                name: "Query".to_owned(),
                description: None,
                fields: {
                    let mut fields = async_graphql::indexmap::IndexMap::new();
                    fields.insert(
                        "__schema".to_string(),
                        MetaField {
                            name: "__schema".to_string(),
                            description: Some(
                                "Access the current type schema of this server.".to_string(),
                            ),
                            args: Default::default(),
                            ty: schema_type,
                            deprecation: Default::default(),
                            cache_control: Default::default(),
                            external: false,
                            requires: None,
                            provides: None,
                            visible: None,
                            compute_complexity: None,
                            resolve: None,
                            transforms: None,
                        },
                    );
                    fields.insert(
                        ::std::borrow::ToOwned::to_owned("userByID"),
                        async_graphql::registry::MetaField {
                            name: ::std::borrow::ToOwned::to_owned("userByID"),
                            description: ::std::option::Option::None,
                            args: {
                                let mut args = async_graphql::indexmap::IndexMap::new();
                                args.insert(
                                    "id".to_owned(),
                                    MetaInputValue {
                                        name: "id".to_owned(),
                                        ty: "ID!".to_string(),
                                        visible: None,
                                        description: Some("User id".to_string()),
                                        is_secret: false,
                                        default_value: None,
                                    },
                                );
                                args
                            },
                            ty: "User".to_owned(),
                            deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                            cache_control: async_graphql::CacheControl {
                                public: true,
                                max_age: 0usize,
                            },
                            external: false,
                            provides: ::std::option::Option::None,
                            requires: ::std::option::Option::None,
                            visible: ::std::option::Option::None,
                            compute_complexity: ::std::option::Option::None,
                            resolve: Some(Resolver {
                                id: Some("id-user".to_string()),
                                r#type: ResolverType::DynamoResolver(DynamoResolver::QueryPKSK {
                                    pk: async_graphql::registry::VariableResolveDefinition::InputTypeName("id".to_string()),
                                    sk: async_graphql::registry::VariableResolveDefinition::InputTypeName("id".to_string())
                                }),
                            }),
                            transforms: None,
                        },
                    );
                    fields.insert(
                        ::std::borrow::ToOwned::to_owned("accountByID"),
                        async_graphql::registry::MetaField {
                            name: ::std::borrow::ToOwned::to_owned("accountByID"),
                            description: ::std::option::Option::None,
                            args: {
                                let mut args = async_graphql::indexmap::IndexMap::new();
                                args.insert(
                                    "id".to_owned(),
                                    MetaInputValue {
                                        name: "id".to_owned(),
                                        ty: "ID!".to_string(),
                                        visible: None,
                                        description: Some("Account ID".to_string()),
                                        is_secret: false,
                                        default_value: None,
                                    },
                                );
                                args
                            },
                            ty: "Account".to_owned(),
                            deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                            cache_control: async_graphql::CacheControl {
                                public: true,
                                max_age: 0usize,
                            },
                            external: false,
                            provides: ::std::option::Option::None,
                            requires: ::std::option::Option::None,
                            visible: ::std::option::Option::None,
                            compute_complexity: ::std::option::Option::None,
                            resolve: Some(Resolver {
                                id: Some("id-account".to_string()),
                                r#type: ResolverType::DynamoResolver(DynamoResolver::QueryPKSK {
                                    pk: async_graphql::registry::VariableResolveDefinition::InputTypeName("id".to_string()),
                                    sk: async_graphql::registry::VariableResolveDefinition::InputTypeName("id".to_string())
                                }),
                            }),
                            transforms: None,
                        },
                    );
                    fields
                },
                cache_control: async_graphql::CacheControl {
                    public: true,
                    max_age: 0usize,
                },
                extends: false,
                keys: ::std::option::Option::None,
                visible: ::std::option::Option::None,
                is_subscription: false,
                rust_typename: "Query".to_owned(),
            }
        },
        "Query",
        "Query",
    );

    let mut file = std::fs::File::create("generated.json").unwrap();
    write!(&mut file, "{:#}", serde_json::to_value(&registry).unwrap()).unwrap();
    file.flush().unwrap();
}
