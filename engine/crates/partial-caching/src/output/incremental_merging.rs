//! Handles merging incremental responses into the OutputStore

use graph_entities::{CompactValue, QueryResponse, QueryResponseNode, ResponseNodeId};
use query_path::QueryPathSegment;

use super::{
    shapes::{ConcreteShape, ObjectShape, OutputShapes},
    store::{ValueId, ValueRecord},
    OutputStore,
};

impl OutputStore {
    pub fn merge_incremental_payload(
        &mut self,
        path: &[&QueryPathSegment],
        mut source: QueryResponse,
        shapes: &OutputShapes,
    ) {
        let (defer_root_shape, dest_value_id) = find_defer_root(self, shapes, path);
        let Some(root_container_id) = source.root else {
            todo!("GB-6966");
        };
        merge_container(
            root_container_id,
            dest_value_id,
            &mut source,
            self,
            ObjectShape::Concrete(defer_root_shape),
        )
    }
}

fn merge_container(
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

    let Some(QueryResponseNode::Container(container)) = source.get_node(container_id) else {
        todo!("GB-6966");
    };

    let fields = container
        .iter()
        .map(|(name, src_id)| {
            let Some(field_shape) = concrete_shape.field(name.as_str()) else {
                todo!("GB-6966");
            };

            (field_shape, *src_id)
        })
        .collect::<Vec<_>>();

    for (field_shape, src_id) in fields {
        let Some(subselection_shape) = field_shape.subselection_shape() else {
            // This must be a leaf field, process it as such
            let field_dest_id = output.field_value_id(object_id, field_shape.index());
            take_leaf_value(source, output, src_id, field_dest_id);
            continue;
        };

        let dest_id = output.field_value_id(object_id, field_shape.index());

        merge_node(src_id, dest_id, source, output, subselection_shape);
    }
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
            merge_container(src_id, dest_id, source, output, subselection_shape);
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
    let ValueRecord::List(dest_ids) = output.value(dest_value_id) else {
        todo!("GB-6966")
    };
    if dest_ids.len() != entry_src_ids.len() {
        todo!("GB-6966")
    }

    for (src_id, dest_id) in entry_src_ids.iter().zip(*dest_ids) {
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

fn find_defer_root<'a>(
    store: &OutputStore,
    shapes: &'a OutputShapes,
    path: &[&QueryPathSegment],
) -> (ConcreteShape<'a>, ValueId) {
    let mut current_shape = shapes.root();
    let Some(mut current_value) = store.root_value() else {
        todo!("GB-6966")
    };

    for segment in path {
        match segment {
            QueryPathSegment::Index(index) => {
                let Some(next_value) = store.index_value_id(current_value, *index) else {
                    todo!("GB-6966")
                };
                current_value = next_value;
            }
            QueryPathSegment::Field(field) => {
                let Some(field) = current_shape.field(field.as_ref()) else {
                    todo!("GB-6966")
                };
                let Some(next_shape) = field.subselection_shape() else {
                    todo!("GB-6966")
                };
                let ValueRecord::Object(object_id) = store.value(current_value) else {
                    todo!("GB-6966")
                };
                current_value = store.field_value_id(*object_id, field.index());
                match next_shape {
                    super::shapes::ObjectShape::Concrete(next_shape) => {
                        current_shape = next_shape;
                    }
                    super::shapes::ObjectShape::Polymorphic(_) => todo!("GB-6949"),
                }
            }
        }
    }

    let ValueRecord::Object(_) = store.value(current_value) else {
        todo!("GB-6966")
    };

    (current_shape, current_value)
}
