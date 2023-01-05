use dynaql_value::ConstValue;
use serde_json::Value;

use crate::{
    QueryResponse, QueryResponseNode, ResponseContainer, ResponseList, ResponseNodeId, ResponseNodeRelation,
    ResponsePrimitive,
};

impl QueryResponse {
    /// Recursive function which take a `serde_json::Value` and transform it into a Node ready to
    /// be used.
    pub fn from_serde_value(&mut self, value: Value) -> ResponseNodeId {
        let a: QueryResponseNode = match value {
            Value::Null => QueryResponseNode::Primitive(ResponsePrimitive::new(ConstValue::Null)),
            Value::Bool(boo) => QueryResponseNode::Primitive(ResponsePrimitive::new(ConstValue::Boolean(boo))),
            Value::Number(n) => QueryResponseNode::Primitive(ResponsePrimitive::new(ConstValue::Number(n))),
            Value::String(s) => QueryResponseNode::Primitive(ResponsePrimitive::new(ConstValue::String(s))),
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
}
