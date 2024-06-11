//! Imports union types
use engine::registry;
use engine_parser::types::TypeKind;

use super::visitor::{Visitor, VisitorContext};

pub struct UnionType;

impl<'a> Visitor<'a> for UnionType {
    fn enter_type_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        type_definition: &'a engine::Positioned<engine_parser::types::TypeDefinition>,
    ) {
        let TypeKind::Union(enum_ty) = &type_definition.node.kind else {
            return;
        };

        let type_name = type_definition.node.name.node.to_string();

        let members = enum_ty
            .members
            .iter()
            .map(|value| value.node.to_string())
            .collect::<Vec<_>>();

        ctx.registry.get_mut().create_type(
            |_| {
                registry::UnionType::new(type_name.clone(), members)
                    .with_description(type_definition.node.description.clone().map(|x| x.node))
                    .into()
            },
            &type_name,
            &type_name,
        );
    }
}
