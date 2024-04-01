use engine::Positioned;
use engine_parser::types::{TypeDefinition, TypeKind};

use super::model_directive::types::{filter, generic, input};
use crate::rules::visitor::{Visitor, VisitorContext};

pub struct MongoDBTypeDirective;

impl<'a> Visitor<'a> for MongoDBTypeDirective {
    fn enter_type_definition(&mut self, ctx: &mut VisitorContext<'a>, type_definition: &'a Positioned<TypeDefinition>) {
        if ctx.registry.borrow().mongodb_configurations.is_empty() {
            return;
        }

        if type_definition.node.extend
            || type_definition
                .node
                .directives
                .iter()
                .any(|directive| directive.is_model())
        {
            return;
        }

        let type_name = type_definition.name.as_str();

        match &type_definition.node.kind {
            TypeKind::Object(object) => {
                let input_type_name = generic::filter_type_name(type_name);
                filter::register_type_input(ctx, object, &input_type_name, std::iter::empty());
                generic::register_array_type(ctx, type_definition.name.as_str(), false);
                generic::register_singular_type(ctx, type_definition.name.as_str());
                filter::register_orderby_input(ctx, object, type_definition.node.name.as_str(), std::iter::empty());
                input::register_type_input(ctx, object, &type_definition.node);
            }
            TypeKind::Enum(_enum) => {
                generic::register_singular_type(ctx, type_name);
                generic::register_array_type(ctx, type_name, false);
                generic::register_update_type(ctx, type_name, false);
                generic::register_update_type(ctx, type_name, true);
            }
            _ => (),
        }
    }
}
