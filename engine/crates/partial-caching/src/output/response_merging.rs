//! Handles merging incremental responses into the OutputStore

use std::collections::HashSet;

use engine_value::Name;
use graph_entities::CompactValue;

use crate::{planning::defers::DeferId, TypeRelationships};

use super::{
    shapes::{ConcreteShape, ObjectShape, OutputShapes},
    store::{ObjectId, ValueId, ValueRecord},
    OutputStore,
};

/// Handles the initial response from the engine
pub fn handle_initial_response(
    response: CompactValue,
    shapes: &OutputShapes,
    root_object: ConcreteShape<'_>,
    type_relationships: &dyn TypeRelationships,
) -> (OutputStore, HashSet<DeferId>) {
    let mut output = OutputStore::default();

    let CompactValue::Object(fields) = response else {
        todo!("GB-6966");
    };

    let dest_root = output.new_value();

    let mut context = MergeContext {
        output: &mut output,
        active_defers: HashSet::new(),
        type_relationships,
        shapes,
    };

    merge_fields_into_value(fields, dest_root, &mut context, ObjectShape::Concrete(root_object));

    let active_defers = context.active_defers;

    (output, active_defers)
}

impl OutputStore {
    pub fn merge_incremental_payload(
        &mut self,
        defer_root_object: ObjectId,
        source: CompactValue,
        shapes: &OutputShapes,
        type_relationships: &dyn TypeRelationships,
    ) -> HashSet<DeferId> {
        let CompactValue::Object(fields) = source else {
            todo!("GB-6966");
        };
        let defer_root_shape = shapes.concrete_object(self.read_object(shapes, defer_root_object).shape_id());

        let mut context = MergeContext {
            output: self,
            active_defers: HashSet::new(),
            type_relationships,
            shapes,
        };

        merge_fields_into_object(fields, defer_root_object, &mut context, defer_root_shape);

        context.active_defers
    }
}

struct MergeContext<'a> {
    output: &'a mut OutputStore,
    active_defers: HashSet<DeferId>,
    type_relationships: &'a dyn TypeRelationships,
    shapes: &'a OutputShapes,
}

fn merge_fields_into_object(
    fields: Vec<(Name, CompactValue)>,
    dest_object_id: ObjectId,
    context: &mut MergeContext<'_>,
    shape: ConcreteShape<'_>,
) {
    let fields = fields
        .into_iter()
        .filter_map(|(name, value)| {
            // If the field is missing from the shape we ignore it.
            // This _could_ be a bug, but it also could just be an implied __typename
            let field_shape = shape.field(name.as_str())?;

            Some((field_shape, value))
        })
        .collect::<Vec<_>>();

    for (field_shape, value) in fields {
        if let Some(defer) = field_shape.defer_id() {
            context.active_defers.insert(defer);
        }

        let Some(subselection_shape) = field_shape.subselection_shape() else {
            // This must be a leaf field, process it as such
            let field_dest_id = context.output.field_value_id(dest_object_id, field_shape.index());
            take_leaf_value(context, value, field_dest_id);
            continue;
        };

        let dest_id = context.output.field_value_id(dest_object_id, field_shape.index());

        merge_value(value, dest_id, context, subselection_shape);
    }
}

fn merge_fields_into_value(
    fields: Vec<(Name, CompactValue)>,
    dest_value_id: ValueId,
    context: &mut MergeContext<'_>,
    object_shape: ObjectShape<'_>,
) {
    let (object_id, concrete_shape) = match context.output.value(dest_value_id) {
        ValueRecord::Unset => {
            let (object_id, concrete_shape) = match object_shape {
                ObjectShape::Concrete(concrete_shape) => {
                    let object_id = context.output.insert_object(concrete_shape);

                    (object_id, concrete_shape)
                }
                ObjectShape::Polymorphic(shape) => {
                    let typename = fields
                        .iter()
                        .find(|(name, _)| name == "__typename")
                        .and_then(|(_, value)| value.as_str());

                    let Some(typename) = typename else { todo!("GB-6966") };

                    let concrete_shape = shape.concrete_shape_for_typename(typename, context.type_relationships);

                    let object_id = context.output.insert_polymorphic_object(concrete_shape, typename);

                    (object_id, concrete_shape)
                }
            };

            context
                .output
                .write_value(dest_value_id, ValueRecord::Object(object_id));

            (object_id, concrete_shape)
        }
        ValueRecord::Object(object_id) => {
            let shape_id = context.output.concrete_shape_of_object(*object_id);
            (*object_id, context.shapes.concrete_object(shape_id))
        }
        _ => todo!("GB-6966"),
    };

    merge_fields_into_object(fields, object_id, context, concrete_shape)
}

fn merge_value(
    value: CompactValue,
    dest_id: ValueId,
    context: &mut MergeContext<'_>,
    subselection_shape: ObjectShape<'_>,
) {
    match value {
        CompactValue::Object(fields) => {
            merge_fields_into_value(fields, dest_id, context, subselection_shape);
        }
        CompactValue::List(items) => merge_list(items, dest_id, context, subselection_shape),
        _ => todo!("GB-6966"),
    }
}

fn merge_list(
    items: Vec<CompactValue>,
    dest_value_id: ValueId,
    context: &mut MergeContext<'_>,
    subselection_shape: ObjectShape<'_>,
) {
    let dest_ids = match context.output.value(dest_value_id) {
        ValueRecord::List(dest_ids) if dest_ids.len() == items.len() => *dest_ids,
        ValueRecord::List(_) => {
            todo!("GB-6966")
        }
        ValueRecord::Unset => {
            let ids = context.output.new_list(items.len());
            context.output.write_value(dest_value_id, ValueRecord::List(ids));
            ids
        }
        _ => {
            todo!("GB-6966")
        }
    };

    for (value, dest_id) in items.into_iter().zip(dest_ids) {
        merge_value(value, dest_id, context, subselection_shape)
    }
}

fn take_leaf_value(context: &mut MergeContext<'_>, value: CompactValue, dest_id: ValueId) {
    let new_value = match value {
        CompactValue::Null => ValueRecord::Null,
        CompactValue::Number(inner) => ValueRecord::Number(inner.clone()),
        CompactValue::String(inner) => ValueRecord::String(inner.into_boxed_str()),
        CompactValue::Boolean(inner) => ValueRecord::Boolean(inner),
        CompactValue::Binary(_) => todo!("do we even use binaries?  not sure we do"),
        CompactValue::Enum(inner) => ValueRecord::String(inner.as_str().into()),
        value @ (CompactValue::List(_) | CompactValue::Object(_)) => ValueRecord::InlineValue(Box::new(value)),
    };

    context.output.write_value(dest_id, new_value);
}
