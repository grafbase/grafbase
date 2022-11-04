use super::visitor::{Visitor, VisitorContext};
use dynaql::Positioned;
use dynaql_parser::types::{FieldDefinition, TypeDefinition};
use dynaql_value::ConstValue;

pub const LENGTH_DIRECTIVE: &str = "length";

pub const MIN_ARGUMENT: &str = "min";
pub const MAX_ARGUMENT: &str = "max";

pub struct LengthDirective;

impl<'a> Visitor<'a> for LengthDirective {
    fn directives(&self) -> String {
        r#"
        directive @length(min: Int, max: Int) on FIELD_DEFINITION
        "#
        .to_string()
    }

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
            .find(|d| d.node.name.node == super::length_directive::LENGTH_DIRECTIVE)
        {
            if !crate::utils::is_type_with_length(&field.node.ty.node) {
                return ctx.report_error(
                    vec![directive.pos],
                    "The @length directive is only accepted on Strings and Lists",
                );
            }

            let arguments = directive
                .node
                .arguments
                .iter()
                .map(|(key, value)| (key.node.as_str(), value));

            let mut allowed_args = [MIN_ARGUMENT, MAX_ARGUMENT];
            let mut remaining_args = allowed_args.len();

            for (name, value) in arguments {
                if let Some(pos) = allowed_args[..remaining_args].iter().position(|x| x == &name) {
                    allowed_args.swap(pos, remaining_args - 1);
                    remaining_args -= 1;

                    if !matches!(value.node, ConstValue::Number(_)) {
                        return ctx.report_error(vec![directive.pos], format!("The {name} argument must be a Number"));
                    }
                } else {
                    return ctx.report_error(
                        vec![directive.pos],
                        format!("The @length directive accepts the `{MIN_ARGUMENT}` and `{MAX_ARGUMENT}` arguments"),
                    );
                }
            }
            if remaining_args == allowed_args.len() {
                ctx.report_error(
                    vec![directive.pos],
                    format!("The @length directive expects at least one of the `{MIN_ARGUMENT}` and `{MAX_ARGUMENT}` arguments"),
                );
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

    #[test]
    fn test_length_wrong_argument_name() {
        let schema = r#"
            type Product @model {
                id: ID!
                name: String @length(foo: 10)
            }
            "#;

        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut LengthDirective, &mut ctx, &schema);

        assert_eq!(ctx.errors.len(), 1);
        assert_eq!(
            ctx.errors.get(0).unwrap().message,
            format!("The @length directive accepts the `{MIN_ARGUMENT}` and `{MAX_ARGUMENT}` arguments"),
        );
    }

    #[test]
    fn test_length_missing_argument() {
        let schema = r#"
            type Product @model {
                id: ID!
                name: String @length
            }
            "#;

        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut LengthDirective, &mut ctx, &schema);

        assert_eq!(ctx.errors.len(), 1);
        assert_eq!(
            ctx.errors.get(0).unwrap().message,
            "The @length directive expects at least one of the `min` and `max` arguments"
        );
    }

    #[test]
    fn test_length_on_id_field() {
        let schema = r#"
            type Product @model {
                id: ID! @length(min: 0, max: 100)
                name: String
            }
            "#;

        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut LengthDirective, &mut ctx, &schema);

        assert_eq!(ctx.errors.len(), 1);
        assert_eq!(
            ctx.errors.get(0).unwrap().message,
            "The @length directive is only accepted on Strings and Lists"
        );
    }

    #[test]
    fn test_length_int_field() {
        let schema = r#"
            type Product @model {
                id: ID!
                name: String!
                category: Int @length(min:0)
            }
            "#;

        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut LengthDirective, &mut ctx, &schema);

        assert_eq!(ctx.errors.len(), 1);
        assert_eq!(
            ctx.errors.get(0).unwrap().message,
            "The @length directive is only accepted on Strings and Lists"
        );
    }

    #[test]
    fn test_length_model_field() {
        let schema = r#"
            type Category @model {
                id: ID!
                name: String!
            }

            type Product @model {
                id: ID!
                name: String!
                category: Category @length(min: 0)
            }
            "#;

        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut LengthDirective, &mut ctx, &schema);

        assert_eq!(ctx.errors.len(), 1);
        assert_eq!(
            ctx.errors.get(0).unwrap().message,
            "The @length directive is only accepted on Strings and Lists"
        );
    }

    #[test]
    fn test_wrong_arg_type() {
        let schema = r#"
            type Product @model {
                id: ID!
                name: String! @length(min: "10")
            }
            "#;

        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut LengthDirective, &mut ctx, &schema);

        assert_eq!(ctx.errors.len(), 1);
        assert_eq!(ctx.errors.get(0).unwrap().message, "The min argument must be a Number",);
    }

    #[test]
    fn test_wrong_arg_name() {
        let schema = r#"
            type Product @model {
                id: ID!
                name: String! @length(value: 10)
            }
            "#;

        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut LengthDirective, &mut ctx, &schema);

        assert_eq!(ctx.errors.len(), 1);
        assert_eq!(
            ctx.errors.get(0).unwrap().message,
            "The @length directive accepts the `min` and `max` arguments"
        );
    }

    #[test]
    fn test_valid() {
        let schema = r#"
            type Product @model {
                id: ID!
                name: String! @length(min: 10, max: 100)
            }
            "#;

        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut LengthDirective, &mut ctx, &schema);

        assert_eq!(ctx.errors.len(), 0, "{}", ctx.errors.get(0).unwrap().message);
    }

    #[test]
    fn test_valid_min() {
        let schema = r#"
            type Product @model {
                id: ID!
                name: String! @length(min: 10)
            }
            "#;

        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut LengthDirective, &mut ctx, &schema);

        assert_eq!(ctx.errors.len(), 0, "{}", ctx.errors.get(0).unwrap().message);
    }

    #[test]
    fn test_valid_max() {
        let schema = r#"
            type Product @model {
                id: ID!
                name: String! @length(max: 10)
            }
            "#;

        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut LengthDirective, &mut ctx, &schema);

        assert_eq!(ctx.errors.len(), 0, "{}", ctx.errors.get(0).unwrap().message);
    }
}
