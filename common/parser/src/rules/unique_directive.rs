use super::visitor::{Visitor, VisitorContext};
use dynaql::Positioned;
use dynaql_parser::types::{FieldDefinition, TypeDefinition};
use serde::{Deserialize, Serialize};

const UNIQUE_DIRECTIVE: &str = "unique";

pub struct UniqueDirective;

#[derive(Debug, Serialize, Deserialize)]
struct Unique {}

impl<'a> Visitor<'a> for UniqueDirective {
    fn directives(&self) -> String {
        r#"
        directive @unique on FIELD_DEFINITION
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
            .find(|d| d.node.name.node == UNIQUE_DIRECTIVE)
        {
            if field.node.ty.node.nullable {
                ctx.report_error(
                    vec![directive.pos],
                    "The @unique directive cannot be used on nullable fields".to_string(),
                );
            }

            if let dynaql_parser::types::BaseType::List(_) = field.node.ty.node.base {
                ctx.report_error(
                    vec![directive.pos],
                    "The @unique directive cannot be used on collections".to_string(),
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
    fn test_not_usable_on_nullable_fields() {
        let schema = r#"
            type Product @model {
                id: ID!
                name: String @unique
            }
            "#;

        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut UniqueDirective, &mut ctx, &schema);

        assert_eq!(ctx.errors.len(), 1);
        assert_eq!(
            ctx.errors.get(0).unwrap().message,
            "The @unique directive cannot be used on nullable fields",
        );
    }

    #[test]
    fn test_usable_on_non_nullable_fields() {
        let schema = r#"
            type Product @model {
                id: ID!
                name: String! @unique
            }
            "#;

        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut UniqueDirective, &mut ctx, &schema);

        assert!(ctx.errors.is_empty());
    }

    #[test]
    fn test_not_usable_on_collection() {
        let schema = r#"
            type Product @model {
                id: ID!
                name: [String]! @unique
            }
            "#;

        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut UniqueDirective, &mut ctx, &schema);

        assert_eq!(ctx.errors.len(), 1);
        assert_eq!(
            ctx.errors.get(0).unwrap().message,
            "The @unique directive cannot be used on collections",
        );
    }
}
