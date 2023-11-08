//! For enum types
//!
//! There is no specialied behavior for enum right now.
//!
//! TODO: Manage deprecation
use engine::{registry, registry::MetaEnumValue};
use engine_parser::types::TypeKind;

use super::{
    deprecated_directive::DeprecatedDirective,
    visitor::{Visitor, VisitorContext},
};

pub struct EnumType;

impl<'a> Visitor<'a> for EnumType {
    fn enter_type_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        type_definition: &'a engine::Positioned<engine_parser::types::TypeDefinition>,
    ) {
        let TypeKind::Enum(enum_ty) = &type_definition.node.kind else {
            return;
        };

        let type_name = type_definition.node.name.node.to_string();

        let values = enum_ty
            .values
            .iter()
            .map(|value| {
                let deprecation = DeprecatedDirective::from_directives(&value.node.directives, ctx);
                MetaEnumValue::new(value.node.value.node.to_string())
                    .with_description(value.node.description.clone().map(|x| x.node))
                    .with_deprecation(deprecation.clone())
            })
            .collect::<Vec<_>>();

        ctx.registry.get_mut().create_type(
            |_| {
                registry::EnumType::new(type_name.clone(), values)
                    .with_description(type_definition.node.description.clone().map(|x| x.node))
                    .into()
            },
            &type_name,
            &type_name,
        );
    }
}
