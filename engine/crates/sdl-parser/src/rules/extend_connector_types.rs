use grafbase_engine::registry::{
    self,
    resolvers::{custom::CustomResolver, Resolver},
    MetaField, MetaInputValue, MetaType,
};
use grafbase_engine_parser::types::TypeKind;

use super::visitor::{Visitor, VisitorContext};
use crate::rules::resolver_directive::ResolverDirective;

pub struct ExtendConnectorTypes;

impl<'a> Visitor<'a> for ExtendConnectorTypes {
    fn enter_type_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        type_definition: &'a grafbase_engine::Positioned<grafbase_engine_parser::types::TypeDefinition>,
    ) {
        let type_name = type_definition.node.name.as_str();
        let TypeKind::Object(object) = &type_definition.node.kind else {
            return;
        };

        if !type_definition.node.extend || matches!(type_name, "Query" | "Mutation") {
            // Query & Mutation extensions are handled in ExtendQueryAndMutationTypes
            return;
        }

        let extended_fields = object
            .fields
            .iter()
            .filter_map(|field| {
                let name = field.node.name.node.to_string();

                let Some(resolver_name) = ResolverDirective::resolver_name(&field.node) else {
                    ctx.report_error(
                        vec![field.pos],
                        format!("Field '{name}' of extended '{type_name}' must hold a `@resolver` directive."),
                    );
                    return None;
                };

                let field = &field.node;

                Some(MetaField {
                    name,
                    description: field.description.clone().map(|x| x.node),
                    args: field
                        .arguments
                        .iter()
                        .map(|argument| {
                            MetaInputValue::new(argument.node.name.to_string(), argument.node.ty.to_string())
                        })
                        .map(|arg| (arg.name.clone(), arg))
                        .collect(),
                    ty: field.ty.clone().node.to_string().into(),
                    resolver: Resolver::CustomResolver(CustomResolver {
                        resolver_name: resolver_name.to_owned(),
                    }),
                    ..MetaField::default()
                })
            })
            .map(|field| (field.name.clone(), field))
            .collect::<Vec<_>>();

        let mut registry = ctx.registry.borrow_mut();
        let Some(MetaType::Object(registry::ObjectType { fields, .. })) = registry.types.get_mut(type_name) else {
            drop(registry);
            ctx.report_error(vec![type_definition.pos], format!("Type '{type_name}' does not exist"));
            return;
        };

        fields.extend(extended_fields);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use grafbase_engine::registry::{self, MetaField, Registry};
    use serde_json as _;

    use crate::{rules::neon_directive::NeonDirective, ConnectorParsers, GraphqlDirective, OpenApiDirective};

    #[test]
    fn test_connector_models_can_be_extended() {
        let output = futures::executor::block_on(crate::parse(
            r#"
        extend schema @openapi(name: "Stripe", namespace: true, schema: "http://example.com")

        extend type StripeCustomer {
            email: String @resolver(name: "email")
        }
        "#,
            &HashMap::new(),
            &FakeConnectorParser,
        ));

        output
            .unwrap()
            .registry
            .types
            .get("StripeCustomer")
            .unwrap()
            .field_by_name("email")
            .expect("StripeCustomer to have an email field after parsing");
    }

    #[rstest::rstest]
    // Technically there's nothing wrong with this first one, but I'd expect it to not work well,
    // so want to make sure it errors
    #[case::extending_native_type(r#"
        extend schema @openapi(name: "Stripe", namespace: true, schema: "http://example.com")

        extend type Foo {
            foo: String! @resolver(name: "hello")
        }
        type Foo {
            bar: String
        }
    "#, &[
        "Type `Foo` is present multiple times."
    ])]
    #[case::extend_missing_type(r#"
        extend schema @openapi(name: "Stripe", namespace: true, schema: "http://example.com")

        extend type Blah {
            foo: String! @resolver(name: "hello")
        }
    "#, &[
        "Type 'Blah' does not exist"
    ])]
    #[case::extend_without_resolver(r#"
        extend schema @openapi(name: "Stripe", namespace: true, schema: "http://example.com")

        extend type StripeCustomer {
            foo: String!
        }
    "#, &["Field 'foo' of extended 'StripeCustomer' must hold a `@resolver` directive."])]
    fn test_parse_result(#[case] schema: &str, #[case] expected_messages: &[&str]) {
        let output = futures::executor::block_on(crate::parse(schema, &HashMap::new(), &FakeConnectorParser));

        let validation_errors = output.unwrap_err().validation_errors().unwrap_or_default();

        let actual_messages = validation_errors
            .iter()
            .map(|error| error.message.as_str())
            .collect::<Vec<_>>();

        assert_eq!(actual_messages.as_slice(), expected_messages);
    }

    struct FakeConnectorParser;

    #[async_trait::async_trait]
    impl ConnectorParsers for FakeConnectorParser {
        async fn fetch_and_parse_openapi(&self, _directive: OpenApiDirective) -> Result<Registry, Vec<String>> {
            let mut registry = Registry::new();
            registry.types.insert(
                "StripeCustomer".into(),
                registry::ObjectType::new(
                    "StripeCustomer",
                    [MetaField {
                        name: "id".into(),
                        ty: "String".into(),
                        ..MetaField::default()
                    }],
                )
                .into(),
            );
            registry.query_root_mut().fields_mut().unwrap().insert(
                "customer".into(),
                MetaField {
                    name: "customer".into(),
                    ty: "StripeCustomer".into(),
                    ..MetaField::default()
                },
            );
            Ok(registry)
        }

        async fn fetch_and_parse_graphql(&self, _directive: GraphqlDirective) -> Result<Registry, Vec<String>> {
            Err(Vec::new())
        }

        async fn fetch_and_parse_neon(&self, _: &NeonDirective) -> Result<Registry, Vec<String>> {
            Err(Vec::new())
        }
    }
}
