use std::collections::{HashMap, HashSet};

use engine_parser::{types::Field, Positioned};

use crate::{visitor::Visitor, VisitorContext};

pub struct UsedFieldsAggregator<'ctx, 'a>(pub &'a mut HashMap<&'ctx str, HashSet<&'ctx str>>);

impl UsedFieldsAggregator<'_, '_> {
    pub fn finalize(&self) -> String {
        let mut out = String::new();
        for (type_name, fields) in self.0.iter() {
            out.push_str(type_name);
            out.push('.');
            for (i, field) in fields.iter().enumerate() {
                if i > 0 {
                    out.push('+');
                }
                out.push_str(field);
            }
            out.push(',');
        }
        out.pop();
        out
    }
}

impl<'ctx, 'a> Visitor<'ctx, registry_v2::Registry> for UsedFieldsAggregator<'ctx, 'a> {
    fn enter_field(&mut self, ctx: &mut VisitorContext<'ctx, registry_v2::Registry>, field: &'ctx Positioned<Field>) {
        if field.node.name.node.starts_with("__") {
            return;
        }

        // Skip introspection fields
        if let Some(parent_type) = ctx.parent_type().filter(|parent| !parent.name().starts_with("__")) {
            self.0
                .entry(parent_type.name())
                .or_default()
                .insert(field.node.name.node.as_str());
        }
    }
}
