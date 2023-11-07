use serde::ser::{SerializeMap, SerializeSeq};
use serde_json::Value;

use super::{InputNodeSelectionSet, Node, ObjectNode, ObjectNodeId, ResponseGraph};

pub struct Input<'a> {
    pub(super) graph: &'a ResponseGraph,
    pub(super) object_node_ids: &'a [ObjectNodeId],
    pub(super) selection_set: &'a InputNodeSelectionSet,
}

impl<'a> Input<'a> {
    pub fn object_node_id(&self) -> ObjectNodeId {
        *self
            .object_node_ids
            .get(0)
            .expect("At least one object node id must be present in a Input.")
    }

    // Guaranteed to be in the same order as the object nodes themselves
    pub fn object_node_ids(&self) -> &[ObjectNodeId] {
        self.object_node_ids
    }
}

struct ObjectNodeProxy<'a> {
    graph: &'a ResponseGraph,
    node: &'a ObjectNode,
    selection_set: &'a InputNodeSelectionSet,
}

impl<'a> serde::Serialize for Input<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.object_node_ids.len()))?;
        for node_id in self.object_node_ids {
            seq.serialize_element(&ObjectNodeProxy {
                graph: self.graph,
                node: &self.graph[*node_id],
                selection_set: self.selection_set,
            })?;
        }
        seq.end()
    }
}

impl<'a> serde::Serialize for ObjectNodeProxy<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.node.fields.len()))?;
        for (field_name, node) in &self.node.fields {
            if let Some(selection) = self.selection_set.field(*field_name) {
                map.serialize_key(&self.graph[selection.input_name])?;
                match node {
                    Node::Null => map.serialize_value(&Value::Null)?,
                    Node::Bool(b) => map.serialize_value(b)?,
                    Node::Number(n) => map.serialize_value(n)?,
                    Node::String(s) => map.serialize_value(s)?,
                    Node::List(nodes) => {
                        map.serialize_value(&ArrayProxy {
                            graph: self.graph,
                            nodes,
                            selection_set: &selection.subselection,
                        })?;
                    }
                    Node::Object(node_id) => map.serialize_value(&ObjectNodeProxy {
                        graph: self.graph,
                        node: &self.graph[*node_id],
                        selection_set: &selection.subselection,
                    })?,
                }
            }
        }
        map.end()
    }
}

struct ArrayProxy<'a> {
    graph: &'a ResponseGraph,
    nodes: &'a Vec<Node>,
    selection_set: &'a InputNodeSelectionSet,
}

impl<'a> serde::Serialize for ArrayProxy<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.nodes.len()))?;
        for node in self.nodes {
            match node {
                Node::Null => seq.serialize_element(&Value::Null)?,
                Node::Bool(b) => seq.serialize_element(b)?,
                Node::Number(n) => seq.serialize_element(n)?,
                Node::String(s) => seq.serialize_element(s)?,
                Node::List(nodes) => seq.serialize_element(&ArrayProxy {
                    graph: self.graph,
                    nodes,
                    selection_set: self.selection_set,
                })?,
                Node::Object(node_id) => seq.serialize_element(&ObjectNodeProxy {
                    graph: self.graph,
                    node: &self.graph[*node_id],
                    selection_set: self.selection_set,
                })?,
            }
        }
        seq.end()
    }
}
