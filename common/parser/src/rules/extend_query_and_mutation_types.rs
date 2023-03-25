use super::visitor::{Visitor, VisitorContext};
use crate::rules::resolver_directive::ResolverDirective;
use dynaql::registry::resolvers::custom::CustomResolver;
use dynaql::registry::resolvers::{Resolver, ResolverType};
use dynaql::registry::transformers::Transformer;
use dynaql::registry::MetaField;
use dynaql_parser::types::TypeKind;
use if_chain::if_chain;

pub struct ExtendQueryAndMutationTypes;

impl<'a> Visitor<'a> for ExtendQueryAndMutationTypes {
    fn enter_type_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        type_definition: &'a dynaql::Positioned<dynaql_parser::types::TypeDefinition>,
    ) {
        if_chain! {
            if let TypeKind::Object(object) = &type_definition.node.kind;
            if ["Query", "Mutation"].contains(&type_definition.node.name.node.as_str());
            then {
                let type_name = type_definition.node.name.node.to_string();
                if !type_definition.node.extend {
                    ctx.report_error(
                        vec![type_definition.pos],
                        format!("Type `{type_name}` can only appear with the `extend` keyword.")
                    );
                    return;
                }
                for field in &object.fields {
                    let name = field.node.name.node.to_string();
                    let Some(resolver_name) = ResolverDirective::resolver_name(&field.node) else {
                        ctx.report_error(
                            vec![field.pos],
                            format!("Field '{name}' of extended '{type_name}' must hold a `@resolver` directive.")
                        );
                        continue;
                    };
                    ctx.queries.push(MetaField {
                        name: name.clone(),
                        description: field.node.description.clone().map(|x| x.node),
                        args: Default::default(),
                        ty: field.node.ty.clone().node.to_string(),
                        deprecation: Default::default(),
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None,
                        visible: None,
                        compute_complexity: None,
                        resolve: Some(Resolver {
                            id: Some(format!("{}_custom_resolver", type_name.to_lowercase())),
                            r#type: ResolverType::CustomResolver(CustomResolver {
                                resolver_name: resolver_name.to_owned(),
                            }),
                        }),
                        edges: Vec::new(),
                        relation: None,
                        transformer: Some(Transformer::JSONSelect {
                            property: name
                        }),
                        plan: None,
                        required_operation: None,
                        auth: None,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::visitor::visit;
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
}
