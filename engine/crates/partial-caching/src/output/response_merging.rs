//! Handles merging incremental responses into the OutputStore

use std::collections::HashSet;

use graph_entities::{CompactValue, QueryResponse, QueryResponseNode, ResponseNodeId};

use crate::{planning::defers::DeferId, TypeRelationships};

use super::{
    shapes::{ConcreteShape, ObjectShape, OutputShapes},
    store::{ObjectId, ValueId, ValueRecord},
    OutputStore,
};

/// Handles the initial response from the engine
pub fn handle_initial_response(
    mut response: QueryResponse,
    shapes: &OutputShapes,
    root_object: ConcreteShape<'_>,
    type_relationships: &dyn TypeRelationships,
) -> (OutputStore, HashSet<DeferId>) {
    let mut output = OutputStore::default();

    let Some(src_root) = response.root else {
        todo!("GB-6966");
    };

    let dest_root = output.new_value();

    let mut context = MergeContext {
        source: &mut response,
        output: &mut output,
        active_defers: HashSet::new(),
        type_relationships,
        shapes,
    };

    merge_container_into_value(src_root, dest_root, &mut context, ObjectShape::Concrete(root_object));

    let active_defers = context.active_defers;

    (output, active_defers)
}

impl OutputStore {
    pub fn merge_incremental_payload(
        &mut self,
        defer_root_object: ObjectId,
        mut source: QueryResponse,
        shapes: &OutputShapes,
        type_relationships: &dyn TypeRelationships,
    ) -> HashSet<DeferId> {
        let Some(root_container_id) = source.root else {
            todo!("GB-6966");
        };
        let defer_root_shape = shapes.concrete_object(self.read_object(shapes, defer_root_object).shape_id());

        let mut context = MergeContext {
            source: &mut source,
            output: self,
            active_defers: HashSet::new(),
            type_relationships,
            shapes,
        };

        merge_container_into_object(root_container_id, defer_root_object, &mut context, defer_root_shape);

        context.active_defers
    }
}

struct MergeContext<'a> {
    source: &'a mut QueryResponse,
    output: &'a mut OutputStore,
    active_defers: HashSet<DeferId>,
    type_relationships: &'a dyn TypeRelationships,
    shapes: &'a OutputShapes,
}

fn merge_container_into_object(
    container_id: ResponseNodeId,
    dest_object_id: ObjectId,
    context: &mut MergeContext<'_>,
    shape: ConcreteShape<'_>,
) {
    let Some(QueryResponseNode::Container(container)) = context.source.get_node(container_id) else {
        todo!("GB-6966");
    };

    let fields = container
        .iter()
        .filter_map(|(name, src_id)| {
            // If the field is missing from the shape we ignore it.
            // This _could_ be a bug, but it also could just be an implied __typename
            let field_shape = shape.field(name.as_str())?;

            Some((field_shape, *src_id))
        })
        .collect::<Vec<_>>();

    for (field_shape, src_id) in fields {
        if let Some(defer) = field_shape.defer_id() {
            context.active_defers.insert(defer);
        }

        let Some(subselection_shape) = field_shape.subselection_shape() else {
            // This must be a leaf field, process it as such
            let field_dest_id = context.output.field_value_id(dest_object_id, field_shape.index());
            take_leaf_value(context, src_id, field_dest_id);
            continue;
        };

        let dest_id = context.output.field_value_id(dest_object_id, field_shape.index());

        merge_node(src_id, dest_id, context, subselection_shape);
    }
}

fn merge_container_into_value(
    container_id: ResponseNodeId,
    dest_value_id: ValueId,
    context: &mut MergeContext<'_>,
    object_shape: ObjectShape<'_>,
) {
    let (object_id, concrete_shape) = match context.output.value(dest_value_id) {
        ValueRecord::Unset => {
            let concrete_shape = match object_shape {
                ObjectShape::Concrete(concrete) => concrete,
                ObjectShape::Polymorphic(shape) => {
                    let typename = context.source.get_node(container_id).and_then(|node| {
                        context
                            .source
                            .get_node(node.as_container()?.child("__typename")?)?
                            .as_str()
                    });

                    let Some(typename) = typename else { todo!("GB-6966") };
                    shape.concrete_shape_for_typename(typename, context.type_relationships)
                }
            };

            let object_id = context.output.insert_object(concrete_shape);
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

    merge_container_into_object(container_id, object_id, context, concrete_shape)
}

fn merge_node(
    src_id: ResponseNodeId,
    dest_id: ValueId,
    context: &mut MergeContext<'_>,
    subselection_shape: ObjectShape<'_>,
) {
    match context.source.get_node(src_id) {
        Some(QueryResponseNode::Container(_)) => {
            merge_container_into_value(src_id, dest_id, context, subselection_shape);
        }
        Some(QueryResponseNode::List(list)) => merge_list(list.iter().collect(), dest_id, context, subselection_shape),
        Some(QueryResponseNode::Primitive(_)) => {
            todo!("GB-6966")
        }
        None => todo!("GB-6966"),
    }
}

fn merge_list(
    entry_src_ids: Vec<ResponseNodeId>,
    dest_value_id: ValueId,
    context: &mut MergeContext<'_>,
    subselection_shape: ObjectShape<'_>,
) {
    let dest_ids = match context.output.value(dest_value_id) {
        ValueRecord::List(dest_ids) if dest_ids.len() == entry_src_ids.len() => *dest_ids,
        ValueRecord::List(_) => {
            todo!("GB-6966")
        }
        ValueRecord::Unset => {
            let ids = context.output.new_list(entry_src_ids.len());
            context.output.write_value(dest_value_id, ValueRecord::List(ids));
            ids
        }
        _ => {
            todo!("GB-6966")
        }
    };

    for (src_id, dest_id) in entry_src_ids.iter().zip(dest_ids) {
        merge_node(*src_id, dest_id, context, subselection_shape)
    }
}

fn take_leaf_value(context: &mut MergeContext<'_>, src_id: ResponseNodeId, dest_id: ValueId) {
    match context.source.get_node_mut(src_id) {
        Some(QueryResponseNode::Primitive(primitive)) => {
            let value = match std::mem::take(&mut primitive.0) {
                CompactValue::Null => ValueRecord::Null,
                CompactValue::Number(inner) => ValueRecord::Number(inner.clone()),
                CompactValue::String(inner) => ValueRecord::String(inner.into_boxed_str()),
                CompactValue::Boolean(inner) => ValueRecord::Boolean(inner),
                CompactValue::Binary(_) => todo!("do we even use binaries?  not sure we do"),
                CompactValue::Enum(inner) => ValueRecord::String(inner.as_str().into()),
                value @ (CompactValue::List(_) | CompactValue::Object(_)) => ValueRecord::InlineValue(Box::new(value)),
            };
            context.output.write_value(dest_id, value);
        }
        _ => {
            todo!("GB-6966");
        }
    }
}
