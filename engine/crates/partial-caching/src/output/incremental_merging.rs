//! Handles merging incremental responses into the OutputStore

use graph_entities::{CompactValue, QueryResponse, QueryResponseNode, ResponseNodeId};

use super::{
    shapes::{ConcreteShape, ObjectShape, OutputShapes},
    store::{ObjectId, ValueId, ValueRecord},
    OutputStore,
};

impl OutputStore {
    pub fn merge_incremental_payload(
        &mut self,
        defer_root_object: ObjectId,
        mut source: QueryResponse,
        shapes: &OutputShapes,
    ) {
        let Some(root_container_id) = source.root else {
            todo!("GB-6966");
        };
        let defer_root_shape = shapes.concrete_object(self.read_object(shapes, defer_root_object).shape_id());

        merge_container_into_object(
            root_container_id,
            defer_root_object,
            &mut source,
            self,
            defer_root_shape,
        )
    }
}

fn merge_container_into_object(
    container_id: ResponseNodeId,
    dest_object_id: ObjectId,
    source: &mut QueryResponse,
    output: &mut OutputStore,
    shape: ConcreteShape<'_>,
) {
    let Some(QueryResponseNode::Container(container)) = source.get_node(container_id) else {
        todo!("GB-6966");
    };

    let fields = container
        .iter()
        .map(|(name, src_id)| {
            let Some(field_shape) = shape.field(name.as_str()) else {
                todo!("GB-6966");
            };

            (field_shape, *src_id)
        })
        .collect::<Vec<_>>();

    for (field_shape, src_id) in fields {
        let Some(subselection_shape) = field_shape.subselection_shape() else {
            // This must be a leaf field, process it as such
            let field_dest_id = output.field_value_id(dest_object_id, field_shape.index());
            take_leaf_value(source, output, src_id, field_dest_id);
            continue;
        };

        let dest_id = output.field_value_id(dest_object_id, field_shape.index());

        merge_node(src_id, dest_id, source, output, subselection_shape);
    }
}

fn merge_container_into_value(
    container_id: ResponseNodeId,
    dest_value_id: ValueId,
    source: &mut QueryResponse,
    output: &mut OutputStore,
    object_shape: ObjectShape<'_>,
) {
    let concrete_shape = match object_shape {
        ObjectShape::Concrete(concrete) => concrete,
        ObjectShape::Polymorphic(_) => {
            // This requires typeinfo from the caching registry, which is missing just now.
            // Will revisit in GB-6949
            todo!("figure out which branch matches based on the __typename")
        }
    };

    let object_id = match output.value(dest_value_id) {
        ValueRecord::Unset => {
            let object_id = output.insert_object(concrete_shape);
            output.write_value(dest_value_id, ValueRecord::Object(object_id));
            object_id
        }
        ValueRecord::Object(object_id) => *object_id,
        _ => todo!("GB-6966"),
    };

    merge_container_into_object(container_id, object_id, source, output, concrete_shape)
}

fn merge_node(
    src_id: ResponseNodeId,
    dest_id: ValueId,
    source: &mut QueryResponse,
    output: &mut OutputStore,
    subselection_shape: ObjectShape<'_>,
) {
    match source.get_node(src_id) {
        Some(QueryResponseNode::Container(_)) => {
            merge_container_into_value(src_id, dest_id, source, output, subselection_shape);
        }
        Some(QueryResponseNode::List(list)) => {
            merge_list(list.iter().collect(), dest_id, source, output, subselection_shape)
        }
        Some(QueryResponseNode::Primitive(_)) => {
            todo!("GB-6966")
        }
        None => todo!("GB-6966"),
    }
}

fn merge_list(
    entry_src_ids: Vec<ResponseNodeId>,
    dest_value_id: ValueId,
    source: &mut QueryResponse,
    output: &mut OutputStore,
    subselection_shape: ObjectShape<'_>,
) {
    let dest_ids = match output.value(dest_value_id) {
        ValueRecord::List(dest_ids) if dest_ids.len() == entry_src_ids.len() => *dest_ids,
        ValueRecord::List(_) => {
            todo!("GB-6966")
        }
        ValueRecord::Unset => {
            let ids = output.new_list(entry_src_ids.len());
            output.write_value(dest_value_id, ValueRecord::List(ids));
            ids
        }
        _ => {
            todo!("GB-6966")
        }
    };

    for (src_id, dest_id) in entry_src_ids.iter().zip(dest_ids) {
        merge_node(*src_id, dest_id, source, output, subselection_shape)
    }
}

pub fn take_leaf_value(source: &mut QueryResponse, output: &mut OutputStore, src_id: ResponseNodeId, dest_id: ValueId) {
    match source.get_node_mut(src_id) {
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
            output.write_value(dest_id, value);
        }
        _ => {
            todo!("GB-6966");
        }
    }
}
