//! This file deals with importing the engines response into the OutputStore

use graph_entities::{CompactValue, QueryResponse, QueryResponseNode, ResponseContainer, ResponseList, ResponseNodeId};

use super::{
    shapes::ObjectShape,
    store::{ValueId, ValueRecord},
    OutputStore, PartitionShape,
};

impl OutputStore {
    pub fn new(response: QueryResponse, shape: PartitionShape<'_>) -> Self {
        let mut output = OutputStore::default();
        let root_object = shape.root_object();

        let Some(root) = response.root else {
            todo!("do something about this");
        };

        let root_field_id = output.new_value();

        match response.get_node(root) {
            Some(QueryResponseNode::Container(container)) => copy_container(
                container,
                &response,
                &mut output,
                root_field_id,
                ObjectShape::Concrete(root_object),
            ),
            _ => todo!("error"),
        }

        output
    }
}

fn copy_container(
    container: &ResponseContainer,
    response: &QueryResponse,
    output: &mut OutputStore,
    dest_value_id: ValueId,
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

    let object_id = output.insert_object(concrete_shape);
    output.write_value(dest_value_id, ValueRecord::Object(object_id));

    for (name, src_id) in container.iter() {
        let Some(field_shape) = concrete_shape.field(name.as_str()) else {
            // TODO: Somethings probably gone wrong if we hit this branch...
            continue;
        };

        // TODO: We need to record whether a field is associated with a defer,
        // and if so record that defer as "active" so we know to merge in cache fields associated
        // with that defer

        let Some(subselection_shape) = field_shape.subselection_shape() else {
            // This must be a leaf field, process it as such
            let field_dest_id = output.field_value_id(object_id, field_shape.index());
            copy_leaf_value(response, output, *src_id, field_dest_id);
            continue;
        };

        let dest_id = output.field_value_id(object_id, field_shape.index());

        copy_node(*src_id, dest_id, response, output, subselection_shape);
    }
}

fn copy_node(
    src_id: ResponseNodeId,
    dest_id: ValueId,
    response: &QueryResponse,
    output: &mut OutputStore,
    subselection_shape: ObjectShape<'_>,
) {
    match response.get_node(src_id) {
        Some(QueryResponseNode::Container(container)) => {
            copy_container(container, response, output, dest_id, subselection_shape);
        }
        Some(QueryResponseNode::List(list)) => copy_list(list, response, output, dest_id, subselection_shape),
        Some(QueryResponseNode::Primitive(_)) => {
            todo!("this definitely looks like an error")
        }
        None => todo!("error?  continue?  not sure"),
    }
}

fn copy_list(
    list: &ResponseList,
    response: &QueryResponse,
    output: &mut OutputStore,
    dest_value_id: ValueId,
    subselection_shape: ObjectShape<'_>,
) {
    let dest_ids = output.new_list(list.len());
    output.write_value(dest_value_id, ValueRecord::List(dest_ids));

    for (src_id, dest_id) in list.iter().zip(dest_ids) {
        copy_node(src_id, dest_id, response, output, subselection_shape)
    }
}

fn copy_leaf_value(response: &QueryResponse, output: &mut OutputStore, src_id: ResponseNodeId, dest_id: ValueId) {
    match response.get_node(src_id) {
        Some(QueryResponseNode::Primitive(primitive)) => {
            let value = match &primitive.0 {
                CompactValue::Null => ValueRecord::Null,
                CompactValue::Number(inner) => ValueRecord::Number(inner.clone()),
                CompactValue::String(inner) => ValueRecord::String(inner.as_str().into()),
                CompactValue::Boolean(inner) => ValueRecord::Boolean(*inner),
                CompactValue::Binary(_) => todo!("do we even use binaries?  not sure we do"),
                CompactValue::Enum(inner) => ValueRecord::String(inner.as_str().into()),
                value @ (CompactValue::List(_) | CompactValue::Object(_)) => {
                    ValueRecord::InlineValue(Box::new(value.clone()))
                }
            };
            output.write_value(dest_id, value);
        }
        _ => {
            // Will revisit this.
            todo!("should probably convert lists and objects into ValueRecord::InlineValue");
        }
    }
}
