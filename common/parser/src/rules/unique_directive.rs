use case::CaseExt;
use dynaql::{
    registry::{Constraint, MetaInputValue, MetaType, Registry},
    Pos, Positioned,
};
use dynaql_parser::types::{BaseType, FieldDefinition, ObjectType};
use dynaql_value::ConstValue;

use crate::registry::names::MetaNames;

use super::{directive::Directive, relations::RelationEngine, visitor::VisitorContext};

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
        let mut fields = vec![UniqueDirectiveField::parse(ctx, model_name, &field.node, directive.pos)];

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
                fields.push(UniqueDirectiveField::parse(
                    ctx,
                    model_name,
                    &model_field.node,
                    argument.pos,
                ));
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
    pub fn parse(
        ctx: &mut VisitorContext<'_>,
        model_name: &str,
        field: &FieldDefinition,
        pos: Pos,
    ) -> UniqueDirectiveField {
        if field.ty.node.nullable {
            ctx.report_error(
                vec![pos],
                format!(
                    "The @unique directive cannot be used with nullable fields, but {} is nullable",
                    field.name.node
                ),
            );
        }

        if let dynaql_parser::types::BaseType::List(_) = field.ty.node.base {
            ctx.report_error(
                vec![pos],
                format!(
                    "The @unique directive cannot be used with collections, but {} is a collection",
                    field.name.node
                ),
            );
        }

        if RelationEngine::get(ctx, model_name, field).is_some() {
            ctx.report_error(
                vec![pos],
                format!(
                    "The @unique directive cannot be used with relations, but {} is a relation",
                    field.name.node
                ),
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

        MetaInputValue::new(self.name.clone(), ty)
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use dynaql::Schema;
    use pretty_assertions::assert_eq;

    use crate::rules::visitor::RuleError;

    macro_rules! assert_validation_error {
        ($schema:literal, $expected_message:literal) => {
            assert_matches!(
                crate::to_registry($schema)
                    .err()
                    .and_then(crate::Error::validation_errors)
                    // We don't care whether there are more errors or not.
                    // It only matters that we find the expected error.
                    .and_then(|errors| errors.into_iter().next()),
                Some(RuleError { message, .. }) => {
                    assert_eq!(message, $expected_message);
                }
            );
        };
    }

    #[test]
    fn test_not_usable_on_nullable_fields() {
        assert_validation_error!(
            r#"
            type Product @model {
                id: ID!
                name: String @unique
            }
            "#,
            "The @unique directive cannot be used with nullable fields, but name is nullable"
        );
    }

    #[test]
    fn test_usable_on_non_nullable_fields() {
        let registry = crate::to_registry(
            r#"
            type Product @model {
                id: ID!
                name: String! @unique
            }
            "#,
        )
        .unwrap();

        insta::assert_snapshot!("sdl", Schema::new(registry).sdl());
    }

    #[test]
    fn test_not_usable_on_collection() {
        assert_validation_error!(
            r#"
            type Product @model {
                id: ID!
                name: [String]! @unique
            }
            "#,
            "The @unique directive cannot be used with collections, but name is a collection"
        );
    }

    #[test]
    fn test_not_usable_on_relations() {
        assert_validation_error!(
            r#"
            type Category @model {
                id: ID!
                product: Product
            }

            type Product @model {
                id: ID!
                name: [String]!
                category: Category! @unique
            }
            "#,
            "The @unique directive cannot be used with relations, but category is a relation"
        );
    }

    #[test]
    fn test_multifield() {
        let registry = crate::to_registry(
            r#"
            type Product @model {
                id: ID!
                productLine: String!
                name: String! @unique(fields: ["productLine"])
            }
            "#,
        )
        .unwrap();

        insta::assert_snapshot!("multifield_sdl", Schema::new(registry).sdl());
    }

    #[test]
    fn test_multifield_not_usable_on_collection() {
        assert_validation_error!(
            r#"
            type Product @model {
                id: ID!
                productLines: [String]!
                name: String! @unique(fields: ["productLines"])
            }
            "#,
            "The @unique directive cannot be used with collections, but productLines is a collection"
        );
    }

    #[test]
    fn test_multifield_not_usable_on_nullable_fields() {
        assert_validation_error!(
            r#"
            type Product @model {
                id: ID!
                productLine: String
                name: String! @unique(fields: ["productLine"])
            }
            "#,
            "The @unique directive cannot be used with nullable fields, but productLine is nullable"
        );
    }

    #[test]
    fn test_referencing_missing_field() {
        assert_validation_error!(
            r#"
            type Product @model {
                id: ID!
                name: String! @unique(fields: ["productLine"])
            }
            "#,
            "The field productLine referenced in the @unique on name doesn't exist on Product"
        );
    }

    #[test]
    fn test_multifield_not_usable_on_relations() {
        assert_validation_error!(
            r#"
            type Category @model {
                id: ID!
                product: Product
            }

            type Product @model {
                id: ID!
                name: String! @unique(fields: ["category"])
                category: Category!
            }
            "#,
            "The @unique directive cannot be used with relations, but category is a relation"
        );
    }

    #[test]
    fn test_malformed_input() {
        assert_validation_error!(
            r#"
            type Product @model {
                id: ID!
                name: String! @unique(fields: [{hello: "there"}])
            }
            "#,
            "The fields argument to @unique must be a list of strings"
        );
        assert_validation_error!(
            r#"
            type Product @model {
                id: ID!
                name: String! @unique(fields: {hello: "there"})
            }
            "#,
            "The fields argument to @unique must be a list of strings"
        );
        assert_validation_error!(
            r#"
            type Product @model {
                id: ID!
                name: String! @unique(other: ["productLine"])
            }
            "#,
            "Unknown argument to @unique: other"
        );
    }
}
