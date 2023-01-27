use case::CaseExt;
use dynaql::{
    registry::{Constraint, MetaInputValue, MetaType, Registry},
    Pos, Positioned,
};
use dynaql_parser::types::{BaseType, FieldDefinition, ObjectType, TypeDefinition};
use dynaql_value::ConstValue;

use crate::registry::names::MetaNames;

use super::{
    directive::Directive,
    visitor::{Visitor, VisitorContext},
};

pub const UNIQUE_DIRECTIVE: &str = "unique";
pub const UNIQUE_FIELDS_ARGUMENT: &str = "fields";

pub struct UniqueDirective {
    model_name: String,
    fields: Vec<UniqueDirectiveField>,
}

struct UniqueDirectiveField {
    name: String,
    ty: BaseType,
}

impl Directive for UniqueDirective {
    fn definition() -> String {
        r#"
        directive @unique(
            "Additional fields to include in this unique index"
            fields: [String!]
        ) on FIELD_DEFINITION
        "#
        .to_string()
    }
}

impl UniqueDirective {
    pub fn parse(
        ctx: &mut VisitorContext<'_>,
        model: &ObjectType,
        model_name: &str,
        field: &Positioned<FieldDefinition>,
    ) -> Option<UniqueDirective> {
        let directive = field
            .node
            .directives
            .iter()
            .find(|directive| directive.node.name.node == UNIQUE_DIRECTIVE)?;

        let field_name = field.node.name.node.to_string();
        let mut fields = vec![UniqueDirectiveField::parse(ctx, &field.node, directive.pos)];

        for (name, argument) in &directive.node.arguments {
            if name.node != UNIQUE_FIELDS_ARGUMENT {
                ctx.report_error(vec![name.pos], format!("Unknown argument to @unique: {}", name.node));
                return None;
            }

            let ConstValue::List(fields_list) = &argument.node else {
                ctx.report_error(vec![argument.pos], "The fields argument to @unique must be a list of strings");
                return None;
            };

            for field in fields_list {
                let ConstValue::String(field) = field else {
                ctx.report_error(vec![argument.pos], "The fields argument to @unique must be a list of strings");
                return None;
            };
                let Some(model_field) = model.fields.iter().find(|f| f.node.name.node == *field) else {
                    ctx.report_error(
                        vec![argument.pos],
                        format!("The field {field} referenced in the @unique on {field_name} doesn't exist on {model_name}"),
                    );
                    return None;
                };
                fields.push(UniqueDirectiveField::parse(ctx, &model_field.node, argument.pos));
            }
        }

        Some(UniqueDirective {
            fields,
            model_name: model_name.to_string(),
        })
    }

    pub fn name(&self) -> String {
        self.fields
            .iter()
            .map(|f| f.name.as_ref())
            .collect::<Vec<_>>()
            .join("And_")
            .to_camel_lowercase()
    }

    pub fn to_constraint(&self) -> Constraint {
        let fields = self.fields.iter().map(|f| f.name.clone()).collect::<Vec<_>>();

        Constraint::unique(self.name(), fields)
    }

    /// Creates the field on a `xByInput` type that looks up an object using this constraint.
    pub fn lookup_by_field(&self, registry: &mut Registry) -> MetaInputValue {
        if self.fields.len() == 1 {
            return self.fields[0].lookup_by_field(false);
        }

        // If we're unique over >1 field we need a nested input type.

        let nested_type_description = self.type_description();
        let nested_type_name = MetaNames::nested_order_by_input(&self.model_name, &self.name());
        registry.create_type(
            |_| MetaType::InputObject {
                name: nested_type_name.clone(),
                description: Some(nested_type_description.clone()),
                input_fields: self
                    .fields
                    .iter()
                    .map(|f| (f.name.clone(), f.lookup_by_field(true)))
                    .collect(),
                visible: None,
                rust_typename: nested_type_name.clone(),
                oneof: false,
            },
            &nested_type_name,
            &nested_type_name,
        );

        MetaInputValue {
            name: self.name(),
            description: Some(nested_type_description),
            ty: nested_type_name,
            default_value: None,
            validators: None,
            visible: None,
            is_secret: false,
        }
    }

    /// The description of our nested input type
    fn type_description(&self) -> String {
        use std::fmt::Write;

        let mut description = String::new();
        write!(&mut description, "Looks up a {} by ", self.model_name).unwrap();
        for field in self.fields.iter().take(self.fields.len() - 1) {
            write!(&mut description, "{}, ", field.name).unwrap();
        }
        write!(&mut description, "and {}", self.fields.last().unwrap().name).unwrap();

        description
    }
}

impl UniqueDirectiveField {
    pub fn parse(ctx: &mut VisitorContext<'_>, field: &FieldDefinition, pos: Pos) -> UniqueDirectiveField {
        if field.ty.node.nullable {
            ctx.report_error(
                vec![pos],
                "The @unique directive cannot be used on nullable fields".to_string(),
            );
        }

        if let dynaql_parser::types::BaseType::List(_) = field.ty.node.base {
            ctx.report_error(
                vec![pos],
                "The @unique directive cannot be used on collections".to_string(),
            );
        }

        UniqueDirectiveField {
            name: field.name.to_string(),
            ty: field.ty.node.base.clone(),
        }
    }

    fn lookup_by_field(&self, required: bool) -> MetaInputValue {
        let mut ty = self.ty.to_string();
        if required {
            ty.push('!');
        }

        MetaInputValue {
            name: self.name.clone(),
            description: None,
            ty,
            default_value: None,
            validators: None,
            visible: None,
            is_secret: false,
        }
    }
}

pub struct UniqueDirectiveVisitor;

impl<'a> Visitor<'a> for UniqueDirectiveVisitor {
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
        visit(&mut UniqueDirectiveVisitor, &mut ctx, &schema);

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
        visit(&mut UniqueDirectiveVisitor, &mut ctx, &schema);

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
        visit(&mut UniqueDirectiveVisitor, &mut ctx, &schema);

        assert_eq!(ctx.errors.len(), 1);
        assert_eq!(
            ctx.errors.get(0).unwrap().message,
            "The @unique directive cannot be used on collections",
        );
    }
}
