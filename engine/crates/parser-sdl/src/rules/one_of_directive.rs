use std::vec;

use engine::{
    indexmap::IndexMap,
    registry::{InputObjectType, MetaDirective, MetaInputValue},
};
use engine_parser::types::TypeKind;
use if_chain::if_chain;
use registry_v2::DirectiveLocation;

use super::{
    directive::Directive,
    visitor::{Visitor, VisitorContext},
};

pub struct OneOfDirective;

pub const ONE_OF_DIRECTIVE: &str = "oneOf";

impl Directive for OneOfDirective {
    fn definition() -> String {
        r"
        directive @oneOf on INPUT_OBJECT
        "
        .to_string()
    }
}

impl<'a> Visitor<'a> for OneOfDirective {
    fn enter_type_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        type_definition: &'a engine::Positioned<engine_parser::types::TypeDefinition>,
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

                ctx.registry.get_mut().create_type(|_| {
                    InputObjectType::new(
                        one_of_type_name.clone(),
                        input.fields.iter().map(|field| {
                            MetaInputValue::new(field.node.name.to_string(), field.node.ty.to_string())
                        })
                    ).with_description(
                        type_definition.node.description.clone().map(|description| description.node),
                    ).with_oneof(true)
                    .into()
                },
                &one_of_type_name, &one_of_type_name);

                let has_one_of = ctx.registry.get_mut().directives.iter().any(|directive| directive.1.name == ONE_OF_DIRECTIVE);

                if !has_one_of {
                    ctx.registry.get_mut().add_directive(MetaDirective {
                        name: ONE_OF_DIRECTIVE.to_string(),
                        description: Some("Indicates that an input object is a oneOf input object".to_string()),
                        locations: vec![DirectiveLocation::InputObject],
                        args: IndexMap::new(),
                        is_repeatable: false,
                    });
                }
            }
        }
    }
}

#[test]
fn test_not_usable_on_nullable_fields() {
    use engine::registry::MetaType;
    use engine_parser::parse_schema;

    use super::visitor::{visit, VisitorContext};

    let schema = r"
        input UserByInput @oneOf {
            id: ID
            email: String
            name: String
        }
    ";

    let schema = parse_schema(schema).unwrap();
    let mut ctx = VisitorContext::new_for_tests(&schema);
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
        .any(|r#type| matches!(r#type, MetaType::InputObject(InputObjectType { oneof: true, .. }))));
}
