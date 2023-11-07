use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};

use crate::response_graph::{FieldName, Node, ObjectNode, ResponseGraph};

pub struct AnyFieldsSeed<'resp> {
    pub(super) response_graph: &'resp mut ResponseGraph,
}

impl<'de, 'resp> DeserializeSeed<'de> for AnyFieldsSeed<'resp> {
    type Value = Vec<(FieldName, Node)>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(AnyFieldsVistor {
            response_graph: self.response_graph,
        })
    }
}

struct AnyFieldsVistor<'resp> {
    response_graph: &'resp mut ResponseGraph,
}

impl<'de, 'resp> Visitor<'de> for AnyFieldsVistor<'resp> {
    type Value = Vec<(FieldName, Node)>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("an object")
    }

    fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut fields = vec![];

        while let Some(key) = visitor.next_key::<&str>()? {
            let field_name = self.response_graph.edges.intern_field_name(key);
            let value = visitor.next_value_seed(AnyNodeSeed {
                response_graph: self.response_graph,
            })?;
            fields.push((field_name, value));
        }

        Ok(fields)
    }
}

struct AnyNodeSeed<'resp> {
    response_graph: &'resp mut ResponseGraph,
}

impl<'de, 'resp> DeserializeSeed<'de> for AnyNodeSeed<'resp> {
    type Value = Node;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(AnyNodeVisitor {
            response_graph: self.response_graph,
        })
    }
}

struct AnyNodeVisitor<'resp> {
    response_graph: &'resp mut ResponseGraph,
}

impl<'de, 'resp> Visitor<'de> for AnyNodeVisitor<'resp> {
    type Value = Node;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a node")
    }

    #[inline]
    fn visit_bool<E>(self, value: bool) -> Result<Node, E> {
        Ok(Node::Bool(value))
    }

    #[inline]
    fn visit_i64<E>(self, value: i64) -> Result<Node, E> {
        Ok(Node::Number(value.into()))
    }

    #[inline]
    fn visit_u64<E>(self, value: u64) -> Result<Node, E> {
        Ok(Node::Number(value.into()))
    }

    #[inline]
    fn visit_f64<E>(self, value: f64) -> Result<Node, E> {
        Ok(serde_json::Number::from_f64(value)
            .map(Node::Number)
            .unwrap_or(Node::Null))
    }

    #[inline]
    fn visit_str<E>(self, value: &str) -> Result<Node, E> {
        Ok(Node::String(value.to_string()))
    }

    #[inline]
    fn visit_string<E>(self, value: String) -> Result<Node, E> {
        Ok(Node::String(value))
    }

    #[inline]
    fn visit_none<E>(self) -> Result<Node, E> {
        Ok(Node::Null)
    }

    #[inline]
    fn visit_some<D>(self, deserializer: D) -> Result<Node, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        AnyNodeSeed {
            response_graph: self.response_graph,
        }
        .deserialize(deserializer)
    }

    #[inline]
    fn visit_unit<E>(self) -> Result<Node, E> {
        Ok(Node::Null)
    }

    #[inline]
    fn visit_seq<V>(self, mut visitor: V) -> Result<Node, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let mut vec = Vec::new();

        while let Some(elem) = visitor.next_element_seed(AnyNodeSeed {
            response_graph: self.response_graph,
        })? {
            vec.push(elem);
        }

        Ok(Node::List(vec))
    }

    fn visit_map<V>(self, mut visitor: V) -> Result<Node, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut fields = vec![];

        while let Some(key) = visitor.next_key::<&str>()? {
            let field_name = self.response_graph.edges.intern_field_name(key);
            let value = visitor.next_value_seed(AnyNodeSeed {
                response_graph: self.response_graph,
            })?;
            fields.push((field_name, value));
        }

        let object_node_id = self.response_graph.push_object(ObjectNode {
            object_id: None,
            fields,
        });
        Ok(Node::Object(object_node_id))
    }
}
