use serde_json::Value;

use crate::{
    CompactValue, QueryResponse, QueryResponseNode, ResponseContainer, ResponseList, ResponseNodeId,
    ResponseNodeRelation, ResponsePrimitive,
};

impl QueryResponse {
    /// Recursive function which take a `serde_json::Value` and transform it into a Node ready to
    /// be used.
    pub fn from_serde_value(&mut self, value: Value) -> ResponseNodeId {
        let a: QueryResponseNode = match value {
            Value::Null => QueryResponseNode::Primitive(ResponsePrimitive::new(CompactValue::Null)),
            Value::Bool(boo) => QueryResponseNode::Primitive(ResponsePrimitive::new(CompactValue::Boolean(boo))),
            Value::Number(n) => QueryResponseNode::Primitive(ResponsePrimitive::new(CompactValue::Number(n))),
            Value::String(s) => QueryResponseNode::Primitive(ResponsePrimitive::new(CompactValue::String(s))),
            Value::Array(val) => {
                let nodes = val
                    .into_iter()
                    .map(|x| self.from_serde_value(x))
                    .collect::<Vec<ResponseNodeId>>();
                let list = ResponseList::with_children(nodes);
                QueryResponseNode::List(list)
            }
            Value::Object(val) => {
                let nodes = val
                    .into_iter()
                    .map(|(key, x)| {
                        (
                            ResponseNodeRelation::not_a_relation(key.into(), None),
                            self.from_serde_value(x),
                        )
                    })
                    .collect::<Vec<(ResponseNodeRelation, ResponseNodeId)>>();
                let container = ResponseContainer::with_children(nodes);
                QueryResponseNode::Container(container)
            }
        };

        self.new_node_unchecked(a)
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
        match self.0.root.as_ref().filter(|node_id| self.0.node_exists(node_id)) {
            Some(node_id) => NodeSerializer { node_id, graph: self.0 }.serialize(serializer),
            None => serializer.serialize_none(),
        }
    }
}

struct NodeSerializer<'a> {
    node_id: &'a ResponseNodeId,
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
                    .children
                    .iter()
                    .filter(|(_, node_id)| self.graph.node_exists(node_id))
                    .map(|(key, value)| {
                        (
                            key.to_string(),
                            NodeSerializer {
                                node_id: value,
                                graph: self.graph,
                            },
                        )
                    }),
            ),
            QueryResponseNode::List(list) => serializer.collect_seq(
                list.children
                    .iter()
                    .filter(|node_id| self.graph.node_exists(node_id))
                    .map(|value| NodeSerializer {
                        node_id: value,
                        graph: self.graph,
                    }),
            ),
            QueryResponseNode::Primitive(primitive) => primitive.0.serialize(serializer),
        }
    }
}
