use case::CaseExt;
use engine::{
    registry::{Constraint, InputObjectType, MetaInputValue, Registry},
    Pos, Positioned,
};
use engine_parser::types::{BaseType, FieldDefinition, ObjectType};
use engine_value::ConstValue;

use super::{directive::Directive, visitor::VisitorContext};
use crate::registry::names::MetaNames;

pub const UNIQUE_DIRECTIVE: &str = "unique";
pub const UNIQUE_FIELDS_ARGUMENT: &str = "fields";

pub struct UniqueDirective {
    model_name: String,
    fields: Vec<UniqueDirectiveField>,
}

struct UniqueDirectiveField {
    name: String,
    mapped_name: Option<String>,
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
                ctx.report_error(
                    vec![argument.pos],
                    "The fields argument to @unique must be a list of strings",
                );
                return None;
            };

            for field in fields_list {
                let ConstValue::String(field) = field else {
                    ctx.report_error(
                        vec![argument.pos],
                        "The fields argument to @unique must be a list of strings",
                    );
                    return None;
                };
                let Some(model_field) = model.fields.iter().find(|f| f.node.name.node == *field) else {
                    ctx.report_error(
                        vec![argument.pos],
                        format!(
                            "The field {field} referenced in the @unique on {field_name} doesn't exist on {model_name}"
                        ),
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
            .map(|f| f.name.as_str())
            .collect::<Vec<&str>>()
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
            |_| {
                InputObjectType::new(
                    nested_type_name.clone(),
                    self.fields.iter().map(|f| f.lookup_by_field(true)),
                )
                .with_description(Some(nested_type_description.clone()))
                .into()
            },
            &nested_type_name,
            &nested_type_name,
        );

        MetaInputValue::new(self.name(), nested_type_name).with_description(nested_type_description)
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
                format!(
                    "The @unique directive cannot be used with nullable fields, but {} is nullable",
                    field.name.node
                ),
            );
        }

        if let engine_parser::types::BaseType::List(_) = field.ty.node.base {
            ctx.report_error(
                vec![pos],
                format!(
                    "The @unique directive cannot be used with collections, but {} is a collection",
                    field.name.node
                ),
            );
        }

        UniqueDirectiveField {
            name: field.name.to_string(),
            mapped_name: field.mapped_name().map(ToString::to_string),
            ty: field.ty.node.base.clone(),
        }
    }

    fn lookup_by_field(&self, required: bool) -> MetaInputValue {
        let mut ty = self.ty.to_string();
        if required {
            ty.push('!');
        }

        MetaInputValue::new(self.name.clone(), ty).with_rename(self.mapped_name.clone())
    }
}
