use std::collections::HashSet;

use engine::{Positioned, QueryPath};
use engine_parser::types::{FieldDefinition, TypeDefinition};
use engine_scalars::{DynamicScalar, PossibleScalar};
use engine_value::ConstValue;
use meta_type_name::MetaTypeName;
use registry_v2::ScalarParser;

use super::{
    model_directive::ModelDirective,
    visitor::{Visitor, VisitorContext},
};

pub const VALUE_ARGUMENT: &str = "value";

pub struct DefaultDirectiveTypes;

const FIELDS_NOT_ALLOWED: &[&str] = &[engine::names::OUTPUT_FIELD_ID];

impl<'a> Visitor<'a> for DefaultDirectiveTypes {
    fn enter_field(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        field: &'a Positioned<FieldDefinition>,
        _parent_type: &'a Positioned<TypeDefinition>,
    ) {
        if let Some(directive) = field
            .node
            .directives
            .iter()
            .find(|d| d.node.name.node == super::default_directive::DEFAULT_DIRECTIVE)
        {
            if ModelDirective::is_model(ctx, &field.node.ty.node) {
                ctx.report_error(
                    vec![directive.pos],
                    "The @default directive is not accepted on fields referring to other models".to_string(),
                );
            }

            if let Some(field) = FIELDS_NOT_ALLOWED
                .iter()
                .copied()
                .find(|field_name| field.node.name.node == *field_name)
            {
                ctx.report_error(
                    vec![directive.pos],
                    format!("The @default directive is not accepted on the `{field}` field"),
                );
            }

            if let Ok(mut arguments) = super::directive::extract_arguments(ctx, directive, &[&[VALUE_ARGUMENT]], None) {
                let default_value = arguments.remove(VALUE_ARGUMENT).unwrap();

                let error = {
                    let ctx_registry = ctx.registry.borrow();
                    is_valid_input_value(
                        &ctx_registry,
                        &field.node.ty.node.to_string(),
                        &default_value,
                        QueryPath::empty().child(field.node.name.node.as_str()),
                    )
                };
                if let Some(err) = error {
                    ctx.report_error(
                        vec![directive.pos],
                        format!("The @default value is of a wrong type: {err}"),
                    );
                }
            }
        }
    }
}

pub fn is_valid_input_value(
    registry: &registry_v1::Registry,
    type_name: &str,
    value: &ConstValue,
    path: QueryPath,
) -> Option<String> {
    match MetaTypeName::create(type_name) {
        MetaTypeName::NonNull(type_name) => match value {
            ConstValue::Null => Some(valid_error(
                &path,
                format!("expected type \"{type_name}\" but found null"),
            )),
            _ => is_valid_input_value(registry, type_name, value, path),
        },
        MetaTypeName::List(type_name) => match value {
            ConstValue::List(elems) => elems
                .iter()
                .enumerate()
                .find_map(|(idx, elem)| is_valid_input_value(registry, type_name, elem, path.clone().child(idx))),
            ConstValue::Null => None,
            _ => is_valid_input_value(registry, type_name, value, path),
        },
        MetaTypeName::Named(type_name) => {
            if let ConstValue::Null = value {
                return None;
            }

            match registry.types.get(type_name)? {
                registry_v1::MetaType::Scalar(scalar) => match scalar.parser {
                    ScalarParser::PassThrough => None,
                    ScalarParser::BestEffort => {
                        if PossibleScalar::is_valid(type_name, value) {
                            None
                        } else {
                            Some(valid_error(&path, format!("expected type \"{type_name}\"")))
                        }
                    }
                },
                registry_v1::MetaType::Enum(enum_type) => {
                    let enum_name = &enum_type.name;
                    match value {
                        ConstValue::Enum(name) => {
                            if enum_type.enum_values.values().all(|value| value.name.as_str() == name) {
                                Some(valid_error(
                                    &path,
                                    format!("enumeration type \"{enum_name}\" does not contain the value \"{name}\""),
                                ))
                            } else {
                                None
                            }
                        }
                        ConstValue::String(name) => {
                            if enum_type.enum_values.values().all(|value| value.name.as_str() == name) {
                                Some(valid_error(
                                    &path,
                                    format!("enumeration type \"{enum_name}\" does not contain the value \"{name}\""),
                                ))
                            } else {
                                None
                            }
                        }
                        _ => Some(valid_error(
                            &path,
                            format!("expected type \"{type_name}\" but got {value}"),
                        )),
                    }
                }
                registry_v1::MetaType::InputObject(input_object) => match value {
                    ConstValue::Object(values) => {
                        if input_object.oneof {
                            if values.len() != 1 {
                                return Some(valid_error(
                                    &path,
                                    "oneOf input objects require exactly one field".to_string(),
                                ));
                            }

                            if let ConstValue::Null = values[0] {
                                return Some(valid_error(
                                    &path,
                                    "oneOf input objects require a non null argument".to_string(),
                                ));
                            }
                        }

                        let mut input_names: HashSet<&str> = values.keys().map(AsRef::as_ref).collect::<HashSet<_>>();

                        for field in input_object.input_fields.values() {
                            input_names.remove::<str>(&field.name);
                            if let Some(value) = values.get::<str>(&field.name) {
                                if let Some(reason) = is_valid_input_value(
                                    registry,
                                    &field.ty.to_string(),
                                    value,
                                    path.clone().child(field.name.as_str()),
                                ) {
                                    return Some(reason);
                                }
                            } else if field.ty.is_non_null() && field.default_value.is_none() {
                                return Some(valid_error(
                                    &path,
                                    format!(
                                        "field \"{}\" of type \"{}\" is required but not provided",
                                        field.name, input_object.name
                                    ),
                                ));
                            }
                        }

                        if let Some(name) = input_names.iter().next() {
                            return Some(valid_error(
                                &path,
                                format!("unknown field \"{name}\" of type \"{}\"", input_object.name),
                            ));
                        }

                        None
                    }
                    _ => None,
                },
                _ => None,
            }
        }
    }
}

fn valid_error(path_node: &QueryPath, msg: String) -> String {
    format!("\"{path_node}\", {msg}")
}

#[cfg(test)]
mod tests {
    use engine_parser::parse_schema;
    use engine_scalars::{PossibleScalar, SDLDefinitionScalar};
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::rules::visitor::visit;

    #[test]
    fn test_default_with_enum_variant() {
        let schema = r"
            type Product {
                id: ID!
                price: Int! @default(value: 0)
                currency: Currency @default(value: USD)
            }

            enum Currency {
                EUR
                USD
                GBP
            }
        ";

        let mut rules = crate::rules::visitor::VisitorNil
            .with(crate::BasicType)
            .with(crate::EnumType)
            .with(crate::ScalarHydratation);

        let schema = format!("{}\n{schema}", PossibleScalar::sdl());
        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new_for_tests(&schema);

        visit(&mut rules, &mut ctx, &schema);
        visit(&mut crate::DefaultDirectiveTypes, &mut ctx, &schema);

        assert_eq!(ctx.errors, vec![]);
    }

    #[rstest::rstest]
    #[case(r#"
        type Product @model {
            id: ID!
            name: String @default(foo: "default")
        }
    "#, &[
        "The @default directive takes a single `value` argument"
    ])]
    #[case(r"
        type Product @model {
            id: ID!
            name: String @default
        }
    ", &[
        "The @default directive takes a single `value` argument"
    ])]
    #[case(r#"
        type Product @model {
            id: ID! @default(value: "default")
            name: String
        }
    "#, &[
        "The @default directive is not accepted on the `id` field"
    ])]
    #[case(r"
        type Category @model {
            id: ID!
            name: String!
        }

        type Product @model {
            id: ID!
            name: String!
            category: Category @default(value: null)
        }
    ", &[
        "The @default directive is not accepted on fields referring to other models"
    ])]
    #[case(r"
        type Product @model {
            id: ID!
            name: String! @default(value: 10)
        }
    ", &[
        "The @default value is of a wrong type: \"name\", expected type \"String\""
    ])]
    #[case(r#"
        type Product @model {
            id: ID!
            name: String @default(value: "foo")
        }
    "#, &[])]
    fn test_parse_result(#[case] schema: &str, #[case] expected_messages: &[&str]) {
        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new_for_tests(&schema);
        visit(&mut DefaultDirectiveTypes, &mut ctx, &schema);

        let actual_messages: Vec<_> = ctx.errors.iter().map(|error| error.message.as_str()).collect();
        assert_eq!(actual_messages.as_slice(), expected_messages);
    }
}
