//! For enum types
//!
//! There is no specialied behavior for enum right now.
//!
//! TODO: Manage deprecation
use super::visitor::{Visitor, VisitorContext};
use dynaql::indexmap::IndexMap;
use dynaql::registry::MetaEnumValue;
use dynaql::registry::MetaType;
use dynaql_parser::types::TypeKind;
use if_chain::if_chain;

pub struct EnumType;

impl<'a> Visitor<'a> for EnumType {
    fn enter_type_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        type_definition: &'a dynaql::Positioned<dynaql_parser::types::TypeDefinition>,
    ) {
        if_chain! {
            if let TypeKind::Enum(enum_ty) = &type_definition.node.kind;
            then {
                let type_name = type_definition.node.name.node.to_string();
                ctx.registry.get_mut().create_type(&mut |_| MetaType::Enum {
                    name: type_name.clone(),
                    description: type_definition.node.description.clone().map(|x| x.node),
                    visible: None,
                    rust_typename: type_name.clone(),
                    enum_values: {
                        let mut values = IndexMap::new();
                        for v in &enum_ty.values {
                            let enum_value = &v.node.value.node;
                            values.insert(
                                enum_value.to_string(),
                                MetaEnumValue {
                                    name: enum_value.to_string(),
                                    description: v.node.description.clone().map(|x| x.node),
                                    deprecation: dynaql::registry::Deprecation::NoDeprecated,
                                    visible: None,
                                }
                                );
                        }
                        values
                    }
                }, &type_name, &type_name);
            }
        }
    }
}
