use super::visitor::{Visitor, VisitorContext};
use dynaql::{
    indexmap::IndexMap,
    registry::{MetaDirective, MetaInputValue, MetaType, __DirectiveLocation},
};
use dynaql_parser::types::TypeKind;
use if_chain::if_chain;
use std::vec;

pub struct OneOfDirective;

pub const ONE_OF_DIRECTIVE: &str = "oneOf";

impl<'a> Visitor<'a> for OneOfDirective {
    fn directives(&self) -> String {
        r#"
        directive @oneOf on INPUT_OBJECT
        "#
        .to_string()
    }

    fn enter_type_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        type_definition: &'a dynaql::Positioned<dynaql_parser::types::TypeDefinition>,
    ) {
        let directives = &type_definition.node.directives;
        if_chain! {
            if let Some(directive) = directives
            .iter()
            .find(|directive| directive.node.name.node == ONE_OF_DIRECTIVE);
            if let TypeKind::InputObject(input) = &type_definition.node.kind;
            then {
                for field in &input.fields {
                    if field.node.ty.to_string().ends_with('!') {
                        ctx.report_error(
                            vec![directive.pos],
                            "oneOf input object fields must be nullable".to_string(),
                        );
                        return;
                    }
                }

                let one_of_type_name = type_definition.node.name.node.to_string();

                ctx.registry.get_mut().create_type(&mut |_| MetaType::InputObject  {
                    name: one_of_type_name.clone(),
                    description: type_definition.node.description.clone().map(|description| description.node),
                    visible: None,
                    rust_typename: one_of_type_name.clone(),
                    input_fields: {
                        let mut input_fields = IndexMap::new();
                        for field in &input.fields {
                            input_fields.insert(
                                field.node.name.to_string(),
                                MetaInputValue {
                                    name: field.node.name.to_string(),
                                    description: None,
                                    ty: field.node.ty.to_string(),
                                    default_value: None,
                                    validators: None,
                                    visible: None,
                                    is_secret: false
                                }
                            );
                        }
                        input_fields
                    },
                    oneof: true,
                }, &one_of_type_name, &one_of_type_name);

                let has_one_of = ctx.registry.get_mut().directives.iter().any(|directive| directive.1.name == ONE_OF_DIRECTIVE);

                if !has_one_of {
                    ctx.registry.get_mut().add_directive(MetaDirective {
                        name: ONE_OF_DIRECTIVE.to_string(),
                        description: Some("Indicates that an input object is a oneOf input object".to_string()),
                        locations: vec![__DirectiveLocation::INPUT_OBJECT],
                        args: IndexMap::new(),
                        is_repeatable: false,
                        visible: Some(|_| true),
                    });
                }
            }
        }
    }
}

#[test]
fn test_not_usable_on_nullable_fields() {
    use super::visitor::{visit, VisitorContext};
    use dynaql_parser::parse_schema;

    let schema = r#"
        input UserByInput @oneOf {
            id: ID
            email: String
            name: String
        }
    "#;

    let schema = parse_schema(schema).unwrap();
    let mut ctx = VisitorContext::new(&schema);
    visit(&mut OneOfDirective, &mut ctx, &schema);
    assert!(ctx
        .registry
        .get_mut()
        .directives
        .iter()
        .any(|directive| directive.1.name == "oneOf"));
    assert!(ctx
        .registry
        .get_mut()
        .types
        .values()
        .any(|r#type| matches!(r#type, MetaType::InputObject { oneof: true, .. })));
}
