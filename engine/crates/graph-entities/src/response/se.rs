use serde_json::Value;

use crate::{
    CompactValue, QueryResponse, QueryResponseNode, ResponseContainer, ResponseList, ResponseNodeId, ResponsePrimitive,
};

impl QueryResponse {
    /// Recursive function which take a `serde_json::Value` and transform it into a Node ready to
    /// be used.
    pub fn from_serde_value(&mut self, value: Value) -> ResponseNodeId {
        match value {
            Value::Null => self.insert_node(ResponsePrimitive::new(CompactValue::Null)),
            Value::Bool(boo) => self.insert_node(ResponsePrimitive::new(CompactValue::Boolean(boo))),
            Value::Number(n) => self.insert_node(ResponsePrimitive::new(CompactValue::Number(n))),
            Value::String(s) => self.insert_node(ResponsePrimitive::new(CompactValue::String(s))),
            Value::Array(val) => {
                let nodes = val
                    .into_iter()
                    .map(|x| self.from_serde_value(x))
                    .collect::<Vec<ResponseNodeId>>();

                self.insert_node(ResponseList::with_children(nodes))
            }
            Value::Object(val) => {
                let nodes = val
                    .into_iter()
                    .map(|(key, x)| (key.into(), self.from_serde_value(x)))
                    .collect::<Vec<_>>();

                self.insert_node(ResponseContainer::with_children(nodes))
            }
        }
    }

    pub fn as_graphql_data(&self) -> GraphQlResponseSerializer<'_> {
        GraphQlResponseSerializer(self)
    }
}

/// A Wrapper around QueryResponse that serialises the nodes for
/// external use.
///
/// The default Serialize on QueryResponse maintains the Graph
/// structure, so is not suitable for sending to users.
pub struct GraphQlResponseSerializer<'a>(pub &'a QueryResponse);

impl serde::Serialize for GraphQlResponseSerializer<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.0.root.filter(|node_id| self.0.node_exists(*node_id)) {
            Some(node_id) => NodeSerializer { node_id, graph: self.0 }.serialize(serializer),
            None => serializer.serialize_none(),
        }
    }
}

struct NodeSerializer<'a> {
    node_id: ResponseNodeId,
    graph: &'a QueryResponse,
}

impl serde::Serialize for NodeSerializer<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let node = self
            .graph
            .get_node(self.node_id)
            .expect("node presence to be checked before NodeSerializer");

        match node {
            QueryResponseNode::Container(container) => serializer.collect_map(
                container
                    .0
                    .iter()
                    .filter(|(_, node_id)| self.graph.node_exists(*node_id))
                    .map(|(key, value)| {
                        (
                            key.as_str(),
                            NodeSerializer {
                                node_id: *value,
                                graph: self.graph,
                            },
                        )
                    }),
            ),
            QueryResponseNode::List(list) => serializer.collect_seq(
                list.0
                    .iter()
                    .filter(|node_id| self.graph.node_exists(**node_id))
                    .map(|value| NodeSerializer {
                        node_id: *value,
                        graph: self.graph,
                    }),
            ),
            QueryResponseNode::Primitive(primitive) => primitive.0.serialize(serializer),
        }
    }
}
