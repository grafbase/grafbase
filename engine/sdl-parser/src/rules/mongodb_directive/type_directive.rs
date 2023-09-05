use grafbase_engine::Positioned;
use grafbase_engine_parser::types::{TypeDefinition, TypeKind};

use super::model_directive::types::{filter, generic, input};
use crate::rules::visitor::{Visitor, VisitorContext};

pub struct MongoDBTypeDirective;

impl<'a> Visitor<'a> for MongoDBTypeDirective {
    fn enter_type_definition(&mut self, ctx: &mut VisitorContext<'a>, r#type: &'a Positioned<TypeDefinition>) {
        if ctx.registry.borrow().mongodb_configurations.is_empty() {
            return;
        }

        if r#type.node.directives.iter().any(|directive| directive.is_model()) {
            return;
        }

        let TypeKind::Object(ref object) = r#type.node.kind else {
            return;
        };

        let input_type_name = generic::filter_type_name(r#type.name.as_str());
        filter::register_type_input(ctx, object, &input_type_name, std::iter::empty());
        generic::register_array_type(ctx, r#type.name.as_str(), false);
        generic::register_singular_type(ctx, r#type.name.as_str());
        filter::register_orderby_input(ctx, object, r#type.node.name.as_str(), std::iter::empty());
        input::register_type_input(ctx, object, &r#type.node);
    }
}
