use std::vec;

use super::visitor::{Visitor, VisitorContext};
use case::CaseExt;
use dynaql::{
    indexmap::IndexMap,
    registry::{MetaInputValue, MetaType},
};
use dynaql_parser::types::TypeKind;
use if_chain::if_chain;

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
            .find(|d| d.node.name.node == ONE_OF_DIRECTIVE);
            if let TypeKind::InputObject(input) = &type_definition.node.kind;
            then {
                let one_of_type_name = type_definition.node.name.node.to_string();
                for field in &input.fields {
                    if field.node.ty.to_string().ends_with("!") {
                        ctx.report_error(
                            vec![directive.pos],
                            "oneOf variants must be nullable".to_string(),
                        );
                        return;
                    }
                    let type_name = format!("{}{}InputVariant", field.node.name, one_of_type_name.to_capitalized());
                    ctx.registry.get_mut().create_type(&mut |_| MetaType::InputObject {
                        name: type_name.clone(),
                        description: None,
                        input_fields: IndexMap::from([
                            (
                                field.node.name.to_string(),
                                MetaInputValue{
                                    name: field.node.name.to_string(),
                                    description: None,
                                    ty: field.node.ty.to_string(),
                                    default_value: None,
                                    visible: None,
                                    is_secret: false
                                }
                            )
                        ]),
                        visible: None,
                        rust_typename: type_name.clone(),
                        oneof: false
                    }, &type_name, &type_name);
                }
                let union_type_name = format!("{one_of_type_name}ByInput");
                ctx.registry.get_mut().create_type(&mut |_| MetaType::Union  {
                    name: union_type_name.clone(),
                    description: type_definition.node.description.clone().map(|x| x.node),
                    visible: None,
                    rust_typename: union_type_name.clone(),
                    possible_types: input.fields.iter().map(|field|
                        format!("{}{}InputVariant", field.node.name, one_of_type_name.to_capitalized())
                    ).collect()
                }, &union_type_name, &union_type_name);
            }
        }
    }
}

#[test]
fn test_not_usable_on_nullable_fields() {
    use super::visitor::{visit, VisitorContext};
    use dynaql_parser::parse_schema;

    let schema = r#"
        input User @oneOf {
            id: ID
            email: String
            name: String
        }
    "#;

    let schema = parse_schema(schema).unwrap();
    let mut ctx = VisitorContext::new(&schema);
    visit(&mut OneOfDirective, &mut ctx, &schema);
    println!("{:#?}", ctx.registry.get_mut().types)
}
