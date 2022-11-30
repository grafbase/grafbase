use std::collections::HashMap;

use dynaql::Pos;
use dynaql_parser::types::ObjectType;

use super::visitor::{Visitor, VisitorContext};

pub struct UniqueObjectFields;

impl<'a> Visitor<'a> for UniqueObjectFields {
    fn enter_object_definition(&mut self, ctx: &mut VisitorContext<'a>, object: &'a ObjectType) {
        let mut field_positions: HashMap<String, Vec<Pos>> = HashMap::with_capacity(object.fields.len());
        for field in &object.fields {
            let name = field.node.name.to_string();
            field_positions.entry(name).or_default().push(field.pos);
        }
        for (field_name, positions) in field_positions {
            if positions.len() > 1 {
                ctx.report_error(
                    positions,
                    format!("Field '{field_name}' cannot be defined multiple times."),
                );
            }
        }
    }
}
