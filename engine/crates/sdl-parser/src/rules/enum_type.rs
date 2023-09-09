//! For enum types
//!
//! There is no specialied behavior for enum right now.
//!
//! TODO: Manage deprecation
use grafbase_engine::{registry, registry::MetaEnumValue};
use grafbase_engine_parser::types::TypeKind;
use if_chain::if_chain;

use super::visitor::{Visitor, VisitorContext};

pub struct EnumType;

impl<'a> Visitor<'a> for EnumType {
    fn enter_type_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        type_definition: &'a grafbase_engine::Positioned<grafbase_engine_parser::types::TypeDefinition>,
    ) {
        if_chain! {
            if let TypeKind::Enum(enum_ty) = &type_definition.node.kind;
            then {
                let type_name = type_definition.node.name.node.to_string();
                ctx.registry.get_mut().create_type(|_| {
                    registry::EnumType::new(
                        type_name.clone(),
                        enum_ty.values.iter().map(|value| {
                            MetaEnumValue::new(value.node.value.node.to_string())
                                .with_description(value.node.description.clone().map(|x| x.node))
                        }))
                    .with_description(
                        type_definition.node.description.clone().map(|x| x.node)
                    ).into()
                },
                &type_name, &type_name);
            }
        }
    }
}
