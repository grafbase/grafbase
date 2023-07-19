use super::visitor::{Visitor, VisitorContext, MUTATION_TYPE, QUERY_TYPE};
use crate::rules::cache_directive::CacheDirective;
use crate::rules::resolver_directive::ResolverDirective;
use dynaql::registry::resolvers::custom::CustomResolver;
use dynaql::registry::resolvers::Resolver;
use dynaql::registry::{MetaField, MetaInputValue};
use dynaql_parser::types::{ObjectType, TypeKind};
use grafbase::auth::Operations;

pub struct ExtendQueryAndMutationTypes;

enum EntryPoint {
    Query,
    Mutation,
}

fn find_entry_point(
    type_definition: &dynaql::Positioned<dynaql_parser::types::TypeDefinition>,
) -> Option<(EntryPoint, &ObjectType)> {
    match &type_definition.node.kind {
        TypeKind::Object(object) if type_definition.node.name.node == QUERY_TYPE => Some((EntryPoint::Query, object)),
        TypeKind::Object(object) if type_definition.node.name.node == MUTATION_TYPE => {
            Some((EntryPoint::Mutation, object))
        }
        _ => None,
    }
}

impl<'a> Visitor<'a> for ExtendQueryAndMutationTypes {
    fn enter_type_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        type_definition: &'a dynaql::Positioned<dynaql_parser::types::TypeDefinition>,
    ) {
        if let Some((entry_point, object)) = find_entry_point(type_definition) {
            let type_name = type_definition.node.name.node.to_string();
            if !type_definition.node.extend {
                ctx.report_error(
                    vec![type_definition.pos],
                    format!("Type `{type_name}` can only appear with the `extend` keyword."),
                );
                return;
            }
            let required_operation = match entry_point {
                EntryPoint::Query => Some(Operations::READ),
                EntryPoint::Mutation => Some(Operations::WRITE),
            };
            for field in &object.fields {
                let name = field.node.name.node.to_string();
                let Some(resolver_name) = ResolverDirective::resolver_name(&field.node) else {
                        ctx.report_error(
                            vec![field.pos],
                            format!("Field '{name}' of extended '{type_name}' must hold a `@resolver` directive.")
                        );
                        continue;
                    };
                let (field_collection, cache_control) = match entry_point {
                    EntryPoint::Query => (&mut ctx.queries, CacheDirective::parse(&field.node.directives)),
                    EntryPoint::Mutation => (&mut ctx.mutations, Default::default()),
                };
                field_collection.push(MetaField {
                    name: name.clone(),
                    mapped_name: None,
                    description: field.node.description.clone().map(|x| x.node),
                    args: field
                        .node
                        .arguments
                        .iter()
                        .map(|argument| {
                            (
                                argument.node.name.to_string(),
                                MetaInputValue::new(argument.node.name.to_string(), argument.node.ty.to_string()),
                            )
                        })
                        .collect(),
                    ty: field.node.ty.clone().node.to_string().into(),
                    deprecation: Default::default(),
                    cache_control,
                    external: false,
                    requires: None,
                    provides: None,
                    visible: None,
                    compute_complexity: None,
                    resolver: Resolver::CustomResolver(CustomResolver {
                        resolver_name: resolver_name.to_owned(),
                    }),
                    edges: Vec::new(),
                    relation: None,
                    required_operation,
                    auth: None,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::visitor::visit;
    use dynaql::CacheControl;
    use dynaql_parser::parse_schema;
    use pretty_assertions::assert_eq;

    #[rstest::rstest]
    #[case(r#"
        type Query {
            foo: String!
        }
    "#, &[
        "Type `Query` can only appear with the `extend` keyword."
    ])]
    #[case(r#"
        extend type Query {
            foo: String!
        }
    "#, &[
        "Field 'foo' of extended 'Query' must hold a `@resolver` directive."
    ])]
    #[case(r#"
        extend type Query {
            foo: String! @resolver(name: "return-foo")
        }
    "#, &[])]
    #[case(r#"
        type Mutation {
            foo: String!
        }
    "#, &[
        "Type `Mutation` can only appear with the `extend` keyword."
    ])]
    #[case(r#"
        extend type Mutation {
            foo: String!
        }
    "#, &[
        "Field 'foo' of extended 'Mutation' must hold a `@resolver` directive."
    ])]
    #[case(r#"
        extend type Mutation {
            foo: String! @resolver(name: "return-foo")
        }
    "#, &[])]
    fn test_parse_result(#[case] schema: &str, #[case] expected_messages: &[&str]) {
        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut ExtendQueryAndMutationTypes, &mut ctx, &schema);

        let actual_messages: Vec<_> = ctx.errors.iter().map(|error| error.message.as_str()).collect();
        assert_eq!(actual_messages.as_slice(), expected_messages);
    }

    #[test]
    fn test_parse_result_with_cache() {
        // prepare
        let schema = r#"
            extend type Query {
                foo: String! @resolver(name: "foo") @cache(maxAge: 60)
            }

            extend type Mutation {
                foo: String! @resolver(name: "foo") @cache(maxAge: 60)
            }
        "#;

        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);

        // act
        visit(&mut ExtendQueryAndMutationTypes, &mut ctx, &schema);

        // assert
        assert!(ctx.errors.is_empty());

        let foo_query = ctx
            .queries
            .iter()
            .find(|query| query.name == "foo")
            .expect("Should find foo query");
        let foo_mutation = ctx
            .mutations
            .iter()
            .find(|mutation| mutation.name == "foo")
            .expect("Should find foo mutation");

        assert_eq!(
            foo_query.cache_control,
            CacheControl {
                max_age: 60,
                ..Default::default()
            }
        );

        assert_eq!(foo_mutation.cache_control, Default::default());
    }
}
